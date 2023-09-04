// NAME Mips Assembler

use crate::args::Args;
use crate::u5::U5;
use crate::u6::U6;
use std::fs::File;
use std::fs::{self, copy};
use std::io::Write;
use std::str;

const R_EXPECTED_ARGS: usize = 3;

enum R_form {
    NONE,
    RD_RS_RT,
    RS,
    RD_RT_SHAMT,
}

pub struct R {
    shamt: U5,
    funct: U6,
    form: R_form,
}

const I_EXPECTED_ARGS: usize = 2;

enum I_form {
    NONE,
    RT_IMM,
}

pub struct I {
    opcode: U6,
    form: I_form,
}

pub fn r_operation(mnemonic: &str) -> Result<R, &'static str> {
    match mnemonic {
        "add" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x20 },
            form: R_form::RD_RS_RT,
        }),
        "sub" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x22 },
            form: R_form::RD_RS_RT,
        }),
        "sll" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x00 },
            form: R_form::RD_RT_SHAMT,
        }),
        "srl" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x02 },
            form: R_form::RD_RT_SHAMT,
        }),
        "xor" => Ok(R {
            shamt: U5 { value: 0 },
            funct: U6 { value: 0x26 },
            form: R_form::RD_RS_RT,
        }),
        _ => Err("Failed to match R-instr mnemonic"),
    }
}

pub fn i_operation(mnemonic: &str) -> Result<I, &'static str> {
    match mnemonic {
        "lui" => Ok(I {
            opcode: U6 { value: 0xf },
            form: I_form::RT_IMM,
        }),
        _ => Err("Failed to match I-instr mnemonic"),
    }
}

pub fn read_file(file_path: &str) -> String {
    fs::read_to_string(file_path).expect("Failed to read the file")
}

pub fn tokenize(raw_text: &str) -> Vec<&str> {
    raw_text.split_whitespace().collect::<Vec<&str>>()
}

pub fn write_int(mut file: &File, data: u32) -> std::io::Result<()> {
    // Create a 32-length buffer
    let mut padded_buffer: [u8; 4] = [0; 4];

    // Convert data into bytes
    let bytes = data.to_be_bytes();

    // Copy bytes into buffer at offset s.t. value is left-padded with 0s
    let copy_index = 4 - bytes.len();
    padded_buffer[copy_index..].copy_from_slice(&bytes);

    // Write to file
    file.write_all(&padded_buffer)
}

#[derive(Debug, PartialEq)]
enum AssemblerState {
    Initial,
    Scanning,
    CollectingRArguments,
    CollectingIArguments,
}

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

fn assemble_r(r_struct: &mut R, r_args: Vec<&str>) -> Result<u32, &'static str> {
    let mut rs: u32 = 0;
    let mut rt: u32 = 0;
    let mut rd: u32 = 0;
    let mut shamt: u32 = 0;

    match r_struct.form {
        R_form::RD_RS_RT => {
            rd = assemble_reg(r_args[1])?;
            rs = assemble_reg(r_args[2])?;
            rt = assemble_reg(r_args[3])?;
            shamt = r_struct.shamt.into();
        }
        R_form::RD_RT_SHAMT => {
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

fn assemble_i(i_struct: &mut I, i_args: Vec<&str>) -> Result<u32, &'static str> {
    let mut rs: u32 = 0;
    let mut rt: u32 = 0;
    let mut imm: u32 = 0;

    match i_struct.form {
        I_form::RT_IMM => {
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

pub fn assemble(args: &Args) -> Result<(), &'static str> {
    let input_fn = &args.input_as;
    let output_fn = &args.output_as;

    let file_contents = read_file(input_fn);
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
        form: R_form::NONE,
    };
    let mut r_args: Vec<&str> = Vec::new();
    let mut i_struct: I = I {
        opcode: U6 { value: 0 },
        form: I_form::NONE,
    };
    let mut i_args: Vec<&str> = Vec::new();

    while tokens.len() > 0 {
        let token = tokens.remove(0);

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

        match state {
            AssemblerState::CollectingRArguments => {
                // "1 + " handles instruction mnemonic being included
                if r_args.len() == 1 + R_EXPECTED_ARGS {
                    let assembled_r = assemble_r(&mut r_struct, r_args.clone())?;
                    if let Err(_) = write_int(&output_file, assembled_r) {
                        return Err("Failed to write to output binary");
                    }

                    state = AssemblerState::Scanning;
                }
            }
            AssemblerState::CollectingIArguments => {
                // "1 + " handles instruction mnemonic being included
                if i_args.len() == 1 + I_EXPECTED_ARGS {
                    let assembled_i = assemble_i(&mut i_struct, i_args.clone())?;
                    if let Err(_) = write_int(&output_file, assembled_i) {
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
