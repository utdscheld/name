use std::fmt;

use dap::types::{ExceptionDetails, ExceptionBreakMode};
use dap::responses::ExceptionInfoResponse;

use crate::mips::REGISTER_NAMES;

#[derive(Debug)]
#[derive(PartialEq, Copy, Clone)]
pub enum ExecutionErrors {
    // The program attempted to access an address that was within a
    // valid range, but was outside the current allocation for that range.
    // This should be treated as a warning, and read out as zero.
    MemoryObviousOverrunAccess { load_address: u32 },
    // The program attempted to read from an area for which no valid range existed.
    MemoryIllegalAccess { load_address: u32 },

    UndefinedInstruction { instruction: u32 },
    // Can also refer to underflow
    IntegerOverflow { rt: usize, rs: usize, value1: u32, value2: u32 },

    SyscallInvalidArugment,

    SyscallInvalidSyscallNumber,

    Event { event: ExecutionEvents }
}

#[derive(Debug)]
#[derive(PartialEq, Copy, Clone)]
pub enum ExecutionEvents {
    // The program is done executing.
    ProgramComplete

    // Eventually instruction/data/etc. breakpoints will go here too
}

impl fmt::Display for ExecutionErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}


pub fn exception_pretty_print(reason: Result<(), ExecutionErrors>) -> ExceptionInfoResponse {
    match reason {
    Ok(()) => ExceptionInfoResponse {
        exception_id: "No exception".into(),
        description: None,
        break_mode: ExceptionBreakMode::Never,
        details: None
    },
    Err(reason) =>  match reason {
        // These events aren't lifted out as exceptions,
        // so a well-formed debug adapter should not attempt to view them
        ExecutionErrors::Event { .. } => ExceptionInfoResponse {
            exception_id: "Execution Event".into(),
            description: None,
            break_mode: ExceptionBreakMode::Never,
            details: None
        },
        ExecutionErrors::MemoryObviousOverrunAccess { load_address } => ExceptionInfoResponse { 
            exception_id: "Buffer Overflow".into(), 
            description: Some("NAME detected a buffer overflow error. You may have attempted an acccess outside the bounds of a heap buffer.".into()), 
            break_mode: ExceptionBreakMode::Always, 
            details: Some(ExceptionDetails { 
                message: Some( format!("Access location: {:x}", load_address)
            ), 
            type_name: None, full_type_name: None, evaluate_name: None, stack_trace: None, inner_exception: None })
        },
        ExecutionErrors::MemoryIllegalAccess { load_address } => 
        ExceptionInfoResponse { 
            exception_id: "Illegal Access".into(), 
            description: Some("The program attempted to access memory that does not exist.".into()), 
            break_mode: ExceptionBreakMode::Always, 
            details: Some(ExceptionDetails { 
                message: Some( format!("Access location: {:x}", load_address)
            ), 
            type_name: None, full_type_name: None, evaluate_name: None, stack_trace: None, inner_exception: None })
        },
        ExecutionErrors::UndefinedInstruction { instruction } =>
        ExceptionInfoResponse { 
            exception_id: "Undefined Instruction".into(), 
            description: Some("The program attempted to execute a MIPS instruction that does not exist.".into()), 
            break_mode: ExceptionBreakMode::Always, 
            details: Some(ExceptionDetails { 
                message: Some( format!("Instruction: {:x}", instruction)
            ), 
            type_name: None, full_type_name: None, evaluate_name: None, stack_trace: None, inner_exception: None })
        },
        ExecutionErrors::IntegerOverflow { rt, rs, value1, value2 } =>
        ExceptionInfoResponse { 
            exception_id: "Integer Overflow".into(), 
            description: Some("The program attempted to perform an integer operation that caused an overflow.".into()), 
            break_mode: ExceptionBreakMode::Always, 
            details: Some(ExceptionDetails { 
                message: Some( format!("rs: {}, value: {:x}\nrt: {}, value: {:x}", REGISTER_NAMES[rs], value1, REGISTER_NAMES[rt], value2)
            ), 
            type_name: None, full_type_name: None, evaluate_name: None, stack_trace: None, inner_exception: None })
        },
            
        _ => unimplemented!("adf"),
    }
    }

   
}