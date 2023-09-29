use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use dap::events::{StoppedEventBody, ExitedEventBody};
use dap::responses::{ReadMemoryResponse, SetExceptionBreakpointsResponse, ThreadsResponse, StackTraceResponse, ScopesResponse, VariablesResponse};
use dap::types::{StoppedEventReason, Thread, StackFrame};
use thiserror::Error;

use dap::prelude::*;

mod mips;
use mips::{Mips, ExecutionErrors};

use base64::{Engine as _, engine::general_purpose};

#[derive(Error, Debug)]
enum MyAdapterError {
  #[error("Unhandled command")]
  UnhandledCommandError,

  #[error("Missing command")]
  MissingCommandError,

  #[error("Command argument error")]
  CommandArgumentError
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

fn main() -> DynResult<()> {
  let mut file = File::create("/tmp/name/name_log.txt")?;
  file.write_all(b"Hello world! I am testing!!\n")?;
  let output = BufWriter::new(std::io::stdout());

  let input = BufReader::new(std::io::stdin());


  let mut server = Server::new(input, output);

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
    supports_restart_request: Some(false),
    supports_exception_options: Some(false),
    supports_value_formatting_options: Some(false),
    supports_exception_info_request: Some(false),
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

      // Reset execution and begin again.
      mips = Default::default();
  
      server.send_event(Event::Initialized)?;

    }

    // Launch does nothing in NAME, since all state was already set up by the time the protocol reached this point.
    Command::Launch(_) => {

      let rsp = req.success(
        ResponseBody::Launch,
      );

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
    
    Command::Next(_) => {
      
      let result = mips.step_one();
      let stopped_event_body = match result {
        Ok(()) | Err(ExecutionErrors::ProgramComplete) => {
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
        Err(execution_error) => {
          StoppedEventBody {
            reason: StoppedEventReason::Exception,
            description: Some("Exception".to_owned()),
            thread_id: Some(0),
            preserve_focus_hint: None,
            text: Some(execution_error.to_string()),
            all_threads_stopped: None,
            hit_breakpoint_ids: None
          }
        }
      };

      let rsp = req.success(
        ResponseBody::Next
      );
      server.respond(rsp)?;

      if result == Err(ExecutionErrors::ProgramComplete) {
        server.send_event(Event::Exited(ExitedEventBody{ exit_code: 0 }))?;
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

    Command::Disconnect(_) => {
      let rsp = req.success(
        ResponseBody::Disconnect
      );
      server.respond(rsp)?;
      break;
    }

    Command::StackTrace(_) => {
      let rsp = req.success(
        ResponseBody::StackTrace(StackTraceResponse{stack_frames: vec![
          StackFrame{
            id: 0,
            name: "MIPS".to_string(),
            source: None,
            line: 0,
            column: 0,
            end_line: Some(0),
            end_column: Some(0),
            can_restart: Some(false),
            instruction_pointer_reference: None,
            module_id: None,
            presentation_hint: None
          }
        ], total_frames: None})
      );
      server.respond(rsp)?;
    }
    
    Command::Scopes(_) => {
      let rsp = req.success(
        ResponseBody::Scopes(ScopesResponse{scopes: vec![]})
      );
      server.respond(rsp)?;
    }

    Command::Variables(_) => {
      let rsp = req.success(
        ResponseBody::Variables(VariablesResponse{variables: vec![]})
      );
      server.respond(rsp)?;
    }

    _ => ()
    // _ => () //Err(Box::new(MyAdapterError::UnhandledCommandError))
  };
}

  Ok(())
}