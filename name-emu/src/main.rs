use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use dap::events::{StoppedEventBody, ExitedEventBody, TerminatedEventBody};
use dap::responses::{ReadMemoryResponse, SetExceptionBreakpointsResponse, ThreadsResponse, StackTraceResponse, ScopesResponse, VariablesResponse, ContinueResponse};
use dap::types::{StoppedEventReason, Thread, StackFrame, Scope, Source, Variable};
use thiserror::Error;

use dap::prelude::*;

mod mips;
use mips::Mips;

mod exception;
use exception::{ExecutionErrors, exception_pretty_print, ExecutionEvents};

mod lineinfo;
use lineinfo::{LineInfo, lineinfo_import};

use base64::{Engine as _, engine::general_purpose};
use std::env;
use std::net::TcpListener;

#[derive(Error, Debug)]
enum MyAdapterError {
  #[error("Unhandled command")]
  UnhandledCommandError,

  #[error("Missing command")]
  MissingCommandError,

  #[error("Command argument error")]
  CommandArgumentError,
  
  #[error("Argument parsing error")]
  ArgumentParsingError
}

type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// fn test_mips() {
//     let mut mips = Mips {
//         regs: [0; 32],
//         floats: [0f32; 32],
//         mult_hi: 0,
//         mult_lo: 0,
//         pc: 0,
//         text_memory: [0x3c080032,
//         0x3c090032,
//         0x3c0a0032,
//         0x3c0b0032,
//         0x3c0c0032,
//         0x3c0d0032,
//         0x3c0e0032,
//         0x014b4820, 0x01ae6022, 0x00108140,
//         0x0017aa82, 0x03197826, 0x3c080032 ,0;47]
//     };
//     // The first instructions load a bunch of registers with 0x500000.
//     for _ in 0..8 {
//         mips.step_one();
//     }
//     /*
//     add $t1, $t2, $t3
//     sub $t4, $t5, $t6
//     sll $s0, $s0, 5
//     srl $s5, $s7, 10
//     xor $t7, $t8, $t9
//     lui $t0, 50
//     */
//     for _ in 0..6 {
//         mips.step_one();
//     }
// }

fn reset_mips(program_data: &[u8]) -> Mips {
  // Reset execution and begin again.
  let mut mips: Mips = Default::default();  

  for (i, byte) in program_data.iter().enumerate() {
    mips.write_b(mips::DOT_TEXT_START_ADDRESS + i as u32, *byte).unwrap();
  }
  mips.stop_address = mips::DOT_TEXT_START_ADDRESS as usize + program_data.len();

  mips
}

fn main() -> DynResult<()> {

  let args_strings: Vec<String> = env::args().collect();

  if args_strings.len() != 5 {
      return Err("USAGE: name-emu [port number] [source file] [object file] [line info file]".into());
  }
  let log_path = std::path::Path::join(env::temp_dir().as_path(), "name_log.txt");
  let mut file = File::create(log_path)?;
  file.write_all(b"NAME Development Log\n")?;


  let port_string = args_strings.get(1).unwrap();
  
  let (in_port, out_port) = if let Ok(port_number) = port_string.parse::<u32>() {

      if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port_number)) {

        let (stream, _) = listener.accept().unwrap();
        (stream.try_clone().unwrap(), stream)
      }
      else {
        println!("Failed to bind port {}", port_number);
        return Err(Box::new(MyAdapterError::ArgumentParsingError));
      }
  }
  else {
    println!("Failed to parse port number");
    return Err(Box::new(MyAdapterError::ArgumentParsingError));
  };

  let program_name = args_strings.get(2).unwrap();

  let program_data = match std::fs::read(args_strings.get(3).unwrap()) {
    Ok(program_data) => program_data,
    Err(why) => {
      println!("Failed to open provided object file. Reason: {}", why);
      return Err(Box::new(MyAdapterError::ArgumentParsingError));      
    }
  };

  let program_lineinfo = match std::fs::read_to_string(args_strings.get(4).unwrap()) {
    Ok(program_lineinfo) => program_lineinfo,
    Err(why) => {
      println!("Failed to open provided line info file. Reason: {}", why);
      return Err(Box::new(MyAdapterError::CommandArgumentError));      
    }
  };
  let lineinfo = lineinfo_import(program_lineinfo)?;
  writeln!(file, "Lineinfo read: {:?}", lineinfo)?;


  let mut server = Server::new(BufReader::new(in_port), BufWriter::new(out_port));

  let capabilities = types::Capabilities {
    supports_configuration_done_request: Some(true),
    supports_function_breakpoints: Some(true),
    supports_conditional_breakpoints: Some(false),
    supports_hit_conditional_breakpoints: Some(false),
    supports_evaluate_for_hovers: Some(false),
    exception_breakpoint_filters: None,
    supports_step_back: Some(false),
    supports_set_variable: Some(false),
    supports_restart_frame: Some(false),
    supports_goto_targets_request: Some(false),
    supports_step_in_targets_request: Some(false),
    supports_completions_request: Some(false),
    completion_trigger_characters: None,
    supports_modules_request: Some(false),
    additional_module_columns: None,
    supported_checksum_algorithms: None,
    supports_restart_request: Some(true),
    supports_exception_options: Some(false),
    supports_value_formatting_options: Some(false),
    supports_exception_info_request: Some(true),
    support_terminate_debuggee: Some(false),
    support_suspend_debuggee: Some(false),
    supports_delayed_stack_trace_loading: Some(false),
    supports_loaded_sources_request: Some(false),
    supports_log_points: Some(false),
    supports_terminate_threads_request: Some(false),
    supports_set_expression: Some(false),
    supports_terminate_request: Some(false),
    supports_data_breakpoints: Some(false),
    supports_read_memory_request: Some(false),
    supports_write_memory_request: Some(false),
    supports_disassemble_request: Some(false),
    supports_cancel_request: Some(false),
    supports_breakpoint_locations_request: Some(false),
    supports_clipboard_context: Some(false),
    supports_stepping_granularity: Some(false),
    supports_instruction_breakpoints: Some(false),
    supports_exception_filter_options: Some(false),
    supports_single_thread_execution_requests: Some(false),
  };

  let mut mips: Mips = Default::default();

loop {
  let req = match server.poll_request()? {
    Some(req) => req,
    None => return Err(Box::new(MyAdapterError::MissingCommandError)),
  };
  writeln!(file, "Request {:?} received", req.command)?;
  writeln!(file)?;
  match req.command {
    Command::Initialize(_) => {
      let rsp = req.success(
        ResponseBody::Initialize(capabilities.clone()),
      );
  
      server.respond(rsp)?;
  
      server.send_event(Event::Initialized)?;

      mips = reset_mips(&program_data);

    }

    // Launch does nothing in NAME, since all state was already set up by the time the protocol reached this point.
    Command::Launch(_) => {

      let rsp = req.success(
        ResponseBody::Launch,
      );
      server.respond(rsp)?;

      let stopped_event_body = StoppedEventBody {
        reason: StoppedEventReason::Step,
        description: None,
        thread_id: Some(0),
        preserve_focus_hint: None,
        text: None,
        all_threads_stopped: None,
        hit_breakpoint_ids: None
      };
      server.send_event(Event::Stopped(stopped_event_body))?;
    }

    Command::WriteMemory(write_mem_args) => {
      let bytes = general_purpose::STANDARD.decode(write_mem_args.data)?;
      // let mut i = 0;
      // for values in bytes.windows(4) {
      //   let word: u32 = (values[0] as u32) << 24 & (values[1] as u32) << 16 & (values[2] as u32) << 8 & values[3] as u32;
        
      //   match mips.write_w(mips::DOT_TEXT + i, word) {
      //     Ok(_) => (),
      //     Err(_) => return Err(Box::new(MyAdapterError::CommandArgumentError))
      //   }

      //   i += 1;
      // }

      let address = match write_mem_args.memory_reference.parse::<u32>() {
        Ok(i) => i,
        Err(_) => return Err(Box::new(MyAdapterError::CommandArgumentError))
      } + match write_mem_args.offset {
        Some(value) => value as u32,
        None => 0
      };

      for (i, byte) in bytes.iter().enumerate() {
        match mips.write_b(address + i as u32, *byte) {
          Ok(_) => (),
          Err(_) => return Err(Box::new(MyAdapterError::CommandArgumentError))
        }
      }
    }

    Command::ReadMemory(ref read_mem_args) => {
      let address = match read_mem_args.memory_reference.parse::<u32>() {
        Ok(i) => i,
        Err(_) => return Err(Box::new(MyAdapterError::CommandArgumentError))
      } + match read_mem_args.offset {
        Some(value) => value as u32,
        None => 0
      };

      let mut out_bytes = vec![];
      let mut response = ReadMemoryResponse {
        address: read_mem_args.memory_reference.clone(),
        unreadable_bytes: None,
        data: None
      };
      
      for i in 0..read_mem_args.count {
        if let Ok(read_byte) = mips.read_b(address + i as u32) {
          out_bytes.push(read_byte);
        }
        else {
          response.unreadable_bytes = Some(read_mem_args.count - i);
          break;
        }
      }
      response.data = Some(general_purpose::STANDARD.encode(out_bytes));

      let rsp = req.success(
        ResponseBody::ReadMemory(response)
      );
  
      server.respond(rsp)?;
    }
    
    Command::Next(_) | Command::StepIn(_) => {
      
      let result = mips.step_one(&mut file);
      let stopped_event_body = match result {
        Ok(()) | Err(ExecutionErrors::Event { event: ExecutionEvents::ProgramComplete }) => {
          StoppedEventBody {
            reason: StoppedEventReason::Step,
            description: None,
            thread_id: Some(0),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: None,
            hit_breakpoint_ids: None
          }
        }
        Err(_) => {
          StoppedEventBody {
            reason: StoppedEventReason::Exception,
            description: None,
            thread_id: Some(0),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: None,
            hit_breakpoint_ids: None
          }
        }
      };

      let rsp = req.success(
        ResponseBody::Next
      );
      server.respond(rsp)?;

      if let Err(ExecutionErrors::Event{event}) = result {
        if event == ExecutionEvents::ProgramComplete {
          server.send_event(Event::Terminated(None))?;
          server.send_event(Event::Exited(ExitedEventBody{ exit_code: 0 }))?;
        }
      }
      else {
        writeln!(file, "{:?}", stopped_event_body)?;
        writeln!(file, "{:?}", mips)?;
        server.send_event(Event::Stopped(stopped_event_body))?;
      }
    }

    Command::SetExceptionBreakpoints(_) => {
      let rsp = req.success(
        ResponseBody::SetExceptionBreakpoints(SetExceptionBreakpointsResponse{breakpoints: None})
      );
      server.respond(rsp)?;
    }

    Command::Threads => {
      let rsp = req.success(
        ResponseBody::Threads(ThreadsResponse{threads: vec![Thread{id: 0, name: "MIPS".to_string()}]})
      );
      server.respond(rsp)?;
    }

    Command::Disconnect(ref disconnect_args) => {

      // If this is a restart sequence, don't die, and instead
      // do nothing

      let rst = disconnect_args.restart;
      let restart = rst.map(serde_json::Value::Bool);

      let terminated_event = TerminatedEventBody {
        restart
      };

      server.send_event(Event::Terminated(Some(terminated_event)))?;

      let rsp = req.success(
        ResponseBody::Disconnect
      );
      server.respond(rsp)?;
      
      if let None | Some(false) = rst {
        break;
      }
    }

    Command::StackTrace(_) => {
      let rsp = req.success(
        ResponseBody::StackTrace(StackTraceResponse{stack_frames: vec![
          StackFrame{
            id: 0,
            name: "mips".to_string(),
            source: Some(Source { name: Some(program_name.to_string()), path: None, source_reference: Some(0), presentation_hint: None, origin: None, sources: None, adapter_data: None, checksums: None }),
            line: lineinfo[&(mips.pc as u32)].line_number as i64,
            column: 0,
            end_line: None,
            end_column: None,
            can_restart: None,
            instruction_pointer_reference: None,
            module_id: None,
            presentation_hint: None
          }
        ], total_frames: Some(3)})
      );
      server.respond(rsp)?;
    }
    
    Command::Scopes(_) => {
      let rsp = req.success(
        ResponseBody::Scopes(ScopesResponse{
          scopes: vec![
            Scope {
              name: "Registers".to_string(),
              presentation_hint: Some(types::ScopePresentationhint::Registers),
              // Notably, the magic 1001 is the only variables reference in this program. It can get the registers
              // I'll probably want a second/third reference for floats etc.
              variables_reference: 1001,
              named_variables: None,
              indexed_variables: None,
              expensive: false,
              source: None,
              line: None,
              column: None,
              end_line: None,
              end_column: None
            }
          ]
        })
      );
      server.respond(rsp)?;
    }

    Command::Variables(ref variables_arguments) => {

      // I'm sure I could collect() this somehow but it's 12:15AM
      let mut registers = Vec::with_capacity(mips.regs.len());


      if variables_arguments.variables_reference == 1001 {
        for (i, reg) in mips.regs.iter().enumerate() {
          registers.push(
            Variable {
              name: mips::REGISTER_NAMES[i].to_string(),
              value: format!("0x{:X}", reg),
              type_field: None,
              presentation_hint: None,
              evaluate_name: None, // But I'm sure this should be something
              variables_reference: 0, // Apparently I should make this 0 for non-nested structs
              named_variables: Some(0),
              indexed_variables: Some(0),
              memory_reference: None // I think this would be neat to implement...
            }
          );
        }
        registers.push(
          Variable {
            name: mips::PC_NAME.to_string(),
            value:format!("0x{:X}", mips.pc),
            type_field: None,
            presentation_hint: None,
            evaluate_name: None, // But I'm sure this should be something
            variables_reference: 0, // Apparently I should make this 0 for non-nested structs
            named_variables: Some(0),
            indexed_variables: Some(0),
            memory_reference: None // I think this would be neat to implement...
          }
        );
      }

      let rsp = req.success(
        ResponseBody::Variables(VariablesResponse{variables: registers})
      );
      server.respond(rsp)?;
    }

    Command::Restart(_) => {
      mips = reset_mips(&program_data);

      let rsp = req.success(
        ResponseBody::Restart
      );
      server.respond(rsp)?;

      let stopped_event_body = StoppedEventBody {
        reason: StoppedEventReason::Step,
        description: None,
        thread_id: Some(0),
        preserve_focus_hint: None,
        text: None,
        all_threads_stopped: None,
        hit_breakpoint_ids: None
      };
      server.send_event(Event::Stopped(stopped_event_body))?;
    }

    Command::ExceptionInfo(_) => {
      let exception_info = exception_pretty_print(mips.prev_ins_result);

      let rsp = req.success(
        ResponseBody::ExceptionInfo(exception_info)
      );

      server.respond(rsp)?;
    }

    Command::Continue(_) => {
      let rsp = req.success(
        ResponseBody::Continue(ContinueResponse{ all_threads_continued: Some(true)})
      );
      server.respond(rsp)?;

      // Keep stepping until something happens...
      loop {
        if let Err(_) = mips.step_one(&mut file) {
          break;
        }
      }
      // OK, what happened?
      let stopped_event_body = match mips.prev_ins_result {
        Ok(()) => unreachable!(), // It's unreachable.
        Err(what_happened) => match what_happened {
          ExecutionErrors::Event{event} => match event {
            ExecutionEvents::ProgramComplete => {
              StoppedEventBody {
                reason: StoppedEventReason::Step,
                description: None,
                thread_id: Some(0),
                preserve_focus_hint: None,
                text: None,
                all_threads_stopped: None,
                hit_breakpoint_ids: None
              }
            }
          },
          _ => { // Some kind of exception occurred...
            StoppedEventBody {
              reason: StoppedEventReason::Exception,
              description: None,
              thread_id: Some(0),
              preserve_focus_hint: None,
              text: None,
              all_threads_stopped: None,
              hit_breakpoint_ids: None
            }
          }
        }
      };
      server.send_event(Event::Stopped(stopped_event_body))?;

      // Whole second match body to figure out what to do about it
      match mips.prev_ins_result {
        Ok(()) => unreachable!(), // It's unreachable.
        Err(what_happened) => match what_happened {
          ExecutionErrors::Event{event} => match event {
            ExecutionEvents::ProgramComplete => {
              server.send_event(Event::Terminated(None))?;
              server.send_event(Event::Exited(ExitedEventBody{ exit_code: 0 }))?;
            }
          },
          _ => { // Some kind of exception occurred...
            () // Don't need to do anything else for now
          }
        }
      }
    }

    _ => ()
    // _ => () //Err(Box::new(MyAdapterError::UnhandledCommandError))
  };
}

  Ok(())
}