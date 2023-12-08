// This file implements the system calls used by MARS.
// The system call number is stored in $v0, which is register 2.

use crate::mips::Mips;
use crate::exception::{ExecutionErrors, ExecutionEvents};
use std::io::Stdin;

pub(crate) fn syscall(mips: &mut Mips, code: u32) -> Result<(), ExecutionErrors> {
    match mips.regs[2] {

        // Print integer. Writes the value in $a0 to the screen
        1 => {
            print!("{}", mips.regs[4]);
            Ok(())
        }
        // Print string. Writes null-terminated string pointed to by $a0 to the screen
        4 => {
            let mut str_vec = vec![];
            let mut i = 0;
            loop {
                let read_char = mips.read_b(mips.regs[4] + i)?;
                if read_char == 0 {
                    break;
                }
                str_vec.push(read_char as char);
                i += 1;
            };
            print!("{}", str_vec.iter().collect::<String>());
            Ok(())
        }
        // Exit. Immediately raise a ProgramComplete error
        10 => {
            Err(ExecutionErrors::Event { event: ExecutionEvents::ProgramComplete })
        }
        // Print char. Writes the value in $a0 as a char to the screen
        11 => {
            if let Some(c) = std::char::from_u32(mips.regs[4]) {
                print!("{}", c);
            }
            Ok(())
        }
        _ => Err(ExecutionErrors::SyscallInvalidSyscallNumber)
    }
}
