/// NAME Mips Assembler
use crate::args::Args;
use crate::u5::U5;
use crate::u6::U6;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::str;

/// Number of expected arguments for R-type instructions
const R_EXPECTED_ARGS: usize = 3;

/// The form of an R-type instruction, specificially
/// which arguments it expects in which order
enum RForm {
    None,
    RdRsRt,
    // Rs,
    RdRtShamt,
}

/// The variable components of an R-type instruction
pub struct R {
    shamt: U5,
    funct: U6,
    form: RForm,
}

/// Number of expected arguments for I-type instructions
const I_EXPECTED_ARGS: usize = 2;

/// The form of an I-type instruction, specifically
/// which arguments it expects in which order
enum IForm {
    None,
    RtImm,
}

/// The variable components of an I-type instruction
pub struct I {
    opcode: U6,
    form: IForm,
}

/// Parses an R-type instruction mnemonic into an [R]
pub fn r_operation(mnemonic: &str) -> Result<R, &'static str> {
    match mnemonic {
        "add" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x20 },
            form: RForm::RdRsRt,
        }),
        "sub" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x22 },
            form: RForm::RdRsRt,
        }),
        "sll" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x00 },
            form: RForm::RdRtShamt,
        }),
        "srl" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x02 },
            form: RForm::RdRtShamt,
        }),
        "xor" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x26 },
            form: RForm::RdRsRt,
        }),
        _ => Err("Failed to match R-instr mnemonic"),
    }
}

/// Parses an I-type instruction mnemonic into an [I]
pub fn i_operation(mnemonic: &str) -> Result<I, &'static str> {
    match mnemonic {
        "lui" => Ok(I {
            opcode: U6 { value: 0xf },
            form: IForm::RtImm,
        }),
        _ => Err("Failed to match I-instr mnemonic"),
    }
}

/// Split a string into meaningful, atomic elements of the MIPS language
pub fn tokenize(raw_text: &str) -> Vec<&str> {
    raw_text.split_whitespace().collect::<Vec<&str>>()
}

/// Write a u32 into a file, zero-padded to 32 bits (4 bytes)
pub fn write_u32(mut file: &File, data: u32) -> std::io::Result<()> {
    const PADDED_LENGTH: usize = 4;

    // Create a 4-length buffer
    let mut padded_buffer: [u8; PADDED_LENGTH] = [0; PADDED_LENGTH];

    // Convert data into bytes
    let bytes: [u8; PADDED_LENGTH] = data.to_be_bytes();

    // Copy bytes into buffer at offset s.t. value is left-padded with 0s
    let copy_index = PADDED_LENGTH - bytes.len();
    padded_buffer[copy_index..].copy_from_slice(&bytes);

    // Write to file
    file.write_all(&padded_buffer)
}

/// Represents the state of the assembler at any given point
#[derive(Debug, PartialEq)]
enum AssemblerState {
    /// State before any processing has occurred
    Initial,
    /// The assembler is in the process of scanning in new tokens
    Scanning,
    /// The assembler has encountered an R-type instruction and
    /// is collecting its arguments before assembling
    CollectingRArguments,
    /// The assembler has encountered an I-type instruction and
    /// is collecting its arguments before assembling
    CollectingIArguments,
}

/// Converts a numbered mnemonic ($t0, $s8, etc) or literal (55, 67, etc) to its integer representation
fn reg_number(mnemonic: &str) -> Result<u32, &'static str> {
    if mnemonic.len() != 3 {
        return Err("Mnemonic out of bounds");
    }
    match mnemonic.chars().nth(2) {
        Some(c) => match c.to_digit(10) {
            Some(digit) => Ok(digit),
            _ => Err("Invalid register index"),
        },
        _ => Err("Malformed mnemonic"),
    }
}

/// Given a register or number, assemble it into its integer representation
fn assemble_reg(mnemonic: &str) -> Result<u32, &'static str> {
    if !mnemonic.starts_with("$") {
        return match mnemonic.parse::<u32>() {
            Ok(v) => Ok(v),
            Err(_) => Err("Failed to parse shamt"),
        };
    }

    // match on everything after $
    match &mnemonic[1..] {
        "zero" => Ok(0),
        "at" => Ok(1),
        "gp" => Ok(28),
        "sp" => Ok(29),
        "fp" => Ok(30),
        "ra" => Ok(31),
        _ => {
            let n = reg_number(mnemonic)?;
            let reg = match mnemonic.chars().nth(1) {
                Some('v') => 2 + n,
                Some('a') => 4 + n,
                Some('t') => {
                    if n <= 7 {
                        8 + n
                    } else {
                        // t8, t9 = 24, 25
                        // 24 - 8 + n
                        16 + n
                    }
                }
                Some('s') => 16 + n,
                _ => 99,
            };
            if reg <= 31 {
                Ok(reg)
            } else {
                Err("Register out of bounds")
            }
        }
    }
}

/// Assembles an R-type instruction
fn assemble_r(r_struct: &mut R, r_args: Vec<&str>) -> Result<u32, &'static str> {
    let mut rs: u32 = 0;
    let mut rt: u32 = 0;
    let mut rd: u32 = 0;
    let mut shamt: u32 = 0;

    match r_struct.form {
        RForm::RdRsRt => {
            rd = assemble_reg(r_args[1])?;
            rs = assemble_reg(r_args[2])?;
            rt = assemble_reg(r_args[3])?;
            shamt = r_struct.shamt.into();
        }
        RForm::RdRtShamt => {
            rd = assemble_reg(r_args[1])?;
            rt = assemble_reg(r_args[2])?;
            shamt = assemble_reg(r_args[3])?
        }
        _ => return Err("Unexpected R_form"),
    };

    let mut funct: u32 = r_struct.funct.into();

    // Mask
    rs &= 0b1_1111;
    rt &= 0b1_1111;
    rd &= 0b1_1111;
    shamt &= 0b1_1111;
    funct &= 0b11_1111;

    // opcode : 31 - 26
    let mut result = 0x000000;

    // rs :     25 - 21
    println!("rs: {}", rs);
    result = (result << 6) | rs;

    // rt :     20 - 16
    println!("rt: {}", rt);
    result = (result << 5) | rt;

    // rd :     15 - 11
    println!("rd: {}", rd);
    result = (result << 5) | rd;

    // shamt : 10 - 6
    println!("shamt: {}", shamt);
    result = (result << 5) | shamt;

    // funct : 5 - 0
    result = (result << 6) | funct;

    println!(
        "0x{:0shortwidth$x} {:0width$b}",
        result,
        result,
        shortwidth = 8,
        width = 32
    );
    Ok(result)
}

/// Assembles an I-type instruction
fn assemble_i(i_struct: &mut I, i_args: Vec<&str>) -> Result<u32, &'static str> {
    let mut rs: u32 = 0;
    let mut rt: u32 = 0;
    let mut imm: u32 = 0;

    match i_struct.form {
        IForm::RtImm => {
            rt = assemble_reg(i_args[1])?;
            imm = assemble_reg(i_args[2])?;
        }
        _ => return Err("Unexpected I_form"),
    };

    let mut opcode: u32 = i_struct.opcode.into();

    // Mask
    rs &= 0b1_1111;
    rt &= 0b1_1111;
    opcode &= 0b11_1111;
    imm &= 0b1111_1111_1111_1111;

    // opcode : 31 - 26
    let mut result = opcode;

    // rs :     25 - 21
    println!("rs: {}", rs);
    result = (result << 5) | rs;

    // rt :     20 - 16
    println!("rt: {}", rt);
    result = (result << 5) | rt;

    // imm :    15 - 0
    println!("imm: {}", imm);
    result = (result << 16) | imm;

    println!(
        "0x{:0shortwidth$x} {:0width$b}",
        result,
        result,
        shortwidth = 8,
        width = 32
    );
    Ok(result)
}

// General assembler entrypoint
pub fn assemble(args: &Args) -> Result<(), &'static str> {
    let input_fn = &args.input_as;
    let output_fn = &args.output_as;

    let file_contents: String;
    match fs::read_to_string(input_fn) {
        Ok(v) => file_contents = v,
        Err(_) => return Err("Failed to read input file contents"),
    };

    let mut tokens = tokenize(&file_contents);

    let output_file: File;
    match File::create(output_fn) {
        Ok(v) => output_file = v,
        Err(e) => return Err("Failed to open output file"),
    }

    let mut state = AssemblerState::Initial;
    let mut r_struct: R = R {
        shamt: U5 { value: 0 },
        funct: U6 { value: 0 },
        form: RForm::None,
    };
    let mut r_args: Vec<&str> = Vec::new();
    let mut i_struct: I = I {
        opcode: U6 { value: 0 },
        form: IForm::None,
    };
    let mut i_args: Vec<&str> = Vec::new();

    // Iterate over all tokens
    while tokens.len() > 0 {
        let token = tokens.remove(0);

        // Scan tokens in
        match state {
            AssemblerState::Initial => match token {
                "main:" => state = AssemblerState::Scanning,
                _ => return Err("Code must begin with 'main' label"),
            },
            AssemblerState::Scanning => match r_operation(&token) {
                Ok(instr_info) => {
                    state = AssemblerState::CollectingRArguments;

                    println!("-----------------------------------");
                    println!(
                        "[R] {} - shamt [{:x}] - funct [{:x}]",
                        token, instr_info.shamt, instr_info.funct
                    );

                    r_struct = instr_info;
                    r_args.clear();
                    r_args.push(token)
                }
                _ => match i_operation(&token) {
                    Ok(instr_info) => {
                        state = AssemblerState::CollectingIArguments;

                        println!("-----------------------------------");
                        println!("[I] {} - opcode [{:x}]", token, instr_info.opcode);

                        i_struct = instr_info;
                        i_args.clear();
                        i_args.push(token)
                    }
                    _ => (),
                },
            },
            AssemblerState::CollectingRArguments => {
                let filtered_token = if token.ends_with(',') {
                    &token[..token.len() - 1]
                } else {
                    token
                };
                // Filter out comma
                r_args.push(filtered_token);
            }
            AssemblerState::CollectingIArguments => {
                let filtered_token = if token.ends_with(',') {
                    &token[..token.len() - 1]
                } else {
                    token
                };
                // Filter out comma
                i_args.push(filtered_token);
            }
        }

        // Try to assemble if args collected
        match state {
            AssemblerState::CollectingRArguments => {
                // "1 + " handles instruction mnemonic being included
                if r_args.len() == 1 + R_EXPECTED_ARGS {
                    let assembled_r = assemble_r(&mut r_struct, r_args.clone())?;
                    if let Err(_) = write_u32(&output_file, assembled_r) {
                        return Err("Failed to write to output binary");
                    }

                    state = AssemblerState::Scanning;
                }
            }
            AssemblerState::CollectingIArguments => {
                // "1 + " handles instruction mnemonic being included
                if i_args.len() == 1 + I_EXPECTED_ARGS {
                    let assembled_i = assemble_i(&mut i_struct, i_args.clone())?;
                    if let Err(_) = write_u32(&output_file, assembled_i) {
                        return Err("Failed to write to output binary");
                    }

                    state = AssemblerState::Scanning;
                }
            }
            _ => (),
        };
    }

    Ok(())
}
