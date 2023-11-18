// This file implements the system calls used by MARS.
// The system call number is stored in $v0, which is register 2.

use crate::mips::Mips;
use crate::exception::{ExecutionErrors, ExecutionEvents};
use std::io::Stdin;

pub(crate) fn syscall(mips: &mut Mips, code: u32) -> Result<(), ExecutionErrors> {
    match mips.regs[2] {

        // Exit. Immediately raise a ProgramComplete error
        0 => {
            Err(ExecutionErrors::Event { event: ExecutionEvents::ProgramComplete })
        }
        // Print integer. Writes the value in $a0 to the screen
        1 => {
            print!("{}", mips.regs[4]);
            Ok(())
        }
        // Print string. Writes null-terminated string pointed to by $a0 to the screen
        4 => {
            let mut i = 0;
            loop {
                let read_char = mips.read_b(mips.regs[4] + i)?;
                if read_char == 0 {
                    break;
                }
                print!("{}", read_char);
                i += 1;
            };
            Ok(())
        }

        _ => Err(ExecutionErrors::SyscallInvalidSyscallNumber)
    }
}