/// NAME Mips Assembler
use crate::args::Args;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::str;

fn mask_u8(n: u8, x: u8) -> u8 {
    n & ((1 << x) - 1)
}

fn mask_u32(n: u32, x: u8) -> u32 {
    n & ((1 << x) - 1)
}

const TEXT_ADDRESS_BASE: u32 = 0x400000;
const MIPS_INSTR_BYTE_WIDTH: u32 = 4;

/// The form of an R-type instruction, specificially
/// which arguments it expects in which order
enum RForm {
    None,
    RdRsRt,
    RdRtShamt,
}

/// The variable components of an R-type instruction
pub struct R {
    shamt: u8,
    funct: u8,
    form: RForm,
}

/// The form of an I-type instruction, specifically
/// which arguments it expects in which order
enum IForm {
    None,
    RtImm,
    RtImmRs,
    RtRsImm,
    RsRtLabel,
}

/// The variable components of an I-type instruction
pub struct I {
    opcode: u8,
    form: IForm,
}

/// The variable component of a J-type instruction
pub struct J {
    opcode: u8,
}

/// Parses a label
pub fn label(token: &str) -> Result<&str, &'static str> {
    match token.strip_suffix(':') {
        Some(s) => Ok(s),
        None => Err("Not label"),
    }
}

/// Parses an R-type instruction mnemonic into an [R]
pub fn r_operation(mnemonic: &str) -> Result<R, &'static str> {
    match mnemonic {
        "add" => Ok(R {
            shamt: 0,
            funct: 0x20,
            form: RForm::RdRsRt,
        }),
        "sub" => Ok(R {
            shamt: 0,
            funct: 0x22,
            form: RForm::RdRsRt,
        }),
        "sll" => Ok(R {
            shamt: 0,
            funct: 0x00,
            form: RForm::RdRtShamt,
        }),
        "srl" => Ok(R {
            shamt: 0,
            funct: 0x02,
            form: RForm::RdRtShamt,
        }),
        "xor" => Ok(R {
            shamt: 0,
            funct: 0x26,
            form: RForm::RdRsRt,
        }),
        _ => Err("Failed to match R-instr mnemonic"),
    }
}

/// Parses an I-type instruction mnemonic into an [I]
pub fn i_operation(mnemonic: &str) -> Result<I, &'static str> {
    match mnemonic {
        "ori" => Ok(I {
            opcode: 0xd,
            form: IForm::RtRsImm,
        }),
        "lb" => Ok(I {
            opcode: 0x20,
            form: IForm::RtImmRs,
        }),
        "lbu" => Ok(I {
            opcode: 0x24,
            form: IForm::RtImmRs,
        }),
        "lh" => Ok(I {
            opcode: 0x21,
            form: IForm::RtImmRs,
        }),
        "lhu" => Ok(I {
            opcode: 0x25,
            form: IForm::RtImmRs,
        }),
        "lw" => Ok(I {
            opcode: 0x23,
            form: IForm::RtImmRs,
        }),
        "ll" => Ok(I {
            opcode: 0x30,
            form: IForm::RtImmRs,
        }),
        "lui" => Ok(I {
            opcode: 0xf,
            form: IForm::RtImm,
        }),
        "sb" => Ok(I {
            opcode: 0x28,
            form: IForm::RtImmRs,
        }),
        "sh" => Ok(I {
            opcode: 0x29,
            form: IForm::RtImmRs,
        }),
        "sw" => Ok(I {
            opcode: 0x2b,
            form: IForm::RtImmRs,
        }),
        "sc" => Ok(I {
            opcode: 0x38,
            form: IForm::RtImmRs,
        }),
        "beq" => Ok(I {
            opcode: 0x4,
            form: IForm::RsRtLabel,
        }),
        "bne" => Ok(I {
            opcode: 0x5,
            form: IForm::RsRtLabel,
        }),
        _ => Err("Failed to match I-instr mnemonic"),
    }
}

/// Parses a J-type instruction mnemonic into a [J]
fn j_operation(mnemonic: &str) -> Result<J, &'static str> {
    match mnemonic {
        "j" => Ok(J { opcode: 0x2 }),
        "jal" => Ok(J { opcode: 0x3 }),
        _ => Err("Failed to match J-instr mnemonic"),
    }
}

/// Split a string into meaningful, atomic elements of the MIPS language
pub fn tokenize(raw_text: &str) -> Vec<&str> {
    // raw_text.split_whitespace().collect::<Vec<&str>>()
    raw_text
        .split(&[',', ' ', '\t', '\r', '\n'][..])
        .filter(|&s| !s.is_empty())
        .collect::<Vec<&str>>()
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
    /// The assembler has encountered a J-type instruction and
    /// is collecting its arguments before assembling
    CollectingJArguments,
}

/// Converts a numbered mnemonic ($t0, $s8, etc) or literal (55, 67, etc) to its integer representation
fn reg_number(mnemonic: &str) -> Result<u8, &'static str> {
    if mnemonic.len() != 3 {
        return Err("Mnemonic out of bounds");
    }

    match mnemonic.chars().nth(2) {
        Some(c) => match c.to_digit(10) {
            Some(digit) => {
                if digit <= 31 {
                    Ok(digit as u8)
                } else {
                    Err("Expected u8")
                }
            }
            _ => Err("Invalid register index"),
        },
        _ => Err("Malformed mnemonic"),
    }
}

/// Given a register or number, assemble it into its integer representation
fn assemble_reg(mnemonic: &str) -> Result<u8, &'static str> {
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
                Some('v') => n + 2,
                Some('a') => n + 4,
                Some('t') => {
                    if n <= 7 {
                        n + 8
                    } else {
                        // t8, t9 = 24, 25
                        // 24 - 8 + n
                        n + 16
                    }
                }
                Some('s') => n + 16,
                _ => {
                    // Catch registers like $0
                    mnemonic.parse::<u8>().unwrap_or(99)
                }
            };
            if reg <= 31 {
                Ok(reg)
            } else {
                Err("Register out of bounds")
            }
        }
    }
}

fn enforce_length(arr: &Vec<&str>, len: usize) -> Result<u32, &'static str> {
    if arr.len() != len {
        Err("Failed length enforcement")
    } else {
        Ok(0)
    }
}

/// Assembles an R-type instruction
fn assemble_r(r_struct: &mut R, r_args: Vec<&str>) -> Result<u32, &'static str> {
    let mut rs: u8;
    let mut rt: u8;
    let mut rd: u8;
    let mut shamt: u8;

    match r_struct.form {
        RForm::RdRsRt => {
            enforce_length(&r_args, 4)?;
            rd = assemble_reg(r_args[1])?;
            rs = assemble_reg(r_args[2])?;
            rt = assemble_reg(r_args[3])?;
            shamt = r_struct.shamt;
        }
        RForm::RdRtShamt => {
            enforce_length(&r_args, 4)?;
            rd = assemble_reg(r_args[1])?;
            rs = 0;
            rt = assemble_reg(r_args[2])?;
            shamt = match r_args[3].parse::<u8>() {
                Ok(v) => v,
                Err(_) => return Err("Failed to parse shamt"),
            }
        }
        _ => return Err("Unexpected R_form"),
    };

    let mut funct = r_struct.funct;

    // Mask
    rs = mask_u8(rs, 5);
    rt &= mask_u8(rt, 5);
    rd &= mask_u8(rd, 5);
    shamt &= mask_u8(shamt, 5);
    funct &= mask_u8(funct, 6);

    // opcode : 31 - 26
    let mut result = 0x000000;

    // rs :     25 - 21
    println!("rs: {}", rs);
    result = (result << 6) | u32::from(rs);

    // rt :     20 - 16
    println!("rt: {}", rt);
    result = (result << 5) | u32::from(rt);

    // rd :     15 - 11
    println!("rd: {}", rd);
    result = (result << 5) | u32::from(rd);

    // shamt : 10 - 6
    println!("shamt: {}", shamt);
    result = (result << 5) | u32::from(shamt);

    // funct : 5 - 0
    result = (result << 6) | u32::from(funct);

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
fn assemble_i(
    i_struct: &mut I,
    i_args: Vec<&str>,
    labels: &mut HashMap<&str, u32>,
    instr_address: u32,
) -> Result<u32, &'static str> {
    let mut rs: u8;
    let mut rt: u8;
    let mut imm: u16;

    match i_struct.form {
        IForm::RtImm => {
            enforce_length(&i_args, 3)?;
            rs = 0;
            rt = assemble_reg(i_args[1])?;
            imm = match i_args[2].parse::<u16>() {
                Ok(v) => v,
                Err(_) => return Err("Failed to parse imm"),
            }
        }
        IForm::RtImmRs => {
            enforce_length(&i_args, 3)?;
            rt = assemble_reg(i_args[1])?;

            if let (Some(rs_index), Some(rs_end)) = (i_args[2].find('('), i_args[2].find(')')) {
                let imm_string = &i_args[2][..rs_index];
                let rs_string = &i_args[2][rs_index + 1..rs_end];

                imm = match imm_string.parse::<u16>() {
                    Ok(v) => v,
                    Err(_) => return Err("Failed to parse imm"),
                };
                rs = assemble_reg(rs_string)?
            } else {
                return Err("Expected operands of the form \"Rt, Imm(Rs)\"");
            }
        }
        IForm::RsRtLabel => {
            enforce_length(&i_args, 4)?;
            rs = assemble_reg(i_args[1])?;
            rt = assemble_reg(i_args[2])?;
            match labels.get(i_args[3]) {
                Some(v) => imm = ((*v) - instr_address) as u16,
                None => return Err("Undeclared label"),
            }
            // imm = match i_args[3].parse::<u16>() {
            //     Ok(v) => {
            //         if v % 4 != 0 {
            //             return Err("Branch position must be word-aligned");
            //         } else {
            //             v / 4
            //         }
            //     }
            //     Err(_) => return Err("Failed to parse imm"),
            // };
        }
        IForm::RtRsImm => {
            enforce_length(&i_args, 4)?;
            rt = assemble_reg(i_args[1])?;
            rs = assemble_reg(i_args[2])?;
            imm = match i_args[3].parse::<u16>() {
                Ok(v) => v,
                Err(_) => return Err("Failed to parse imm"),
            };
        }
        _ => return Err("Unexpected I_form"),
    };

    let mut opcode = i_struct.opcode;

    // Mask
    println!("Masking rs");
    rs = mask_u8(rs, 5);
    println!("Masking rt");
    rt = mask_u8(rt, 5);
    println!("Masking opcode");
    opcode = mask_u8(opcode, 6);
    // No need to mask imm, it's already a u16

    // opcode : 31 - 26
    let mut result: u32 = opcode.into();

    // rs :     25 - 21
    println!("rs: {}", rs);
    result = (result << 5) | u32::from(rs);

    // rt :     20 - 16
    println!("rt: {}", rt);
    result = (result << 5) | u32::from(rt);

    // imm :    15 - 0
    println!("imm: {}", imm);
    result = (result << 16) | u32::from(imm);

    println!(
        "0x{:0shortwidth$x} {:0width$b}",
        result,
        result,
        shortwidth = 8,
        width = 32
    );
    Ok(result)
}

/// Assembles a J-type instruction
fn assemble_j(
    j_struct: &mut J,
    j_args: Vec<&str>,
    labels: &mut HashMap<&str, u32>,
) -> Result<u32, &'static str> {
    enforce_length(&j_args, 2)?;

    let jump_address: u32 = labels[j_args[1]];
    println!("Masking jump address");
    println!("Jump address original: {}", jump_address);
    let mut masked_jump_address = mask_u32(jump_address, 26);
    println!("Jump address masked: {}", masked_jump_address);
    if jump_address != masked_jump_address {
        panic!("Tried to assemble illegal jump address");
    }

    // Byte-align jump address
    masked_jump_address >>= 2;

    let mut opcode = j_struct.opcode;

    // Mask
    println!("Masking opcode");
    opcode = mask_u8(opcode, 6);
    // No need to mask imm, it's already a u16

    // opcode : 31 - 26
    let mut result: u32 = opcode.into();

    // imm :    25 - 0
    println!("imm: {}", masked_jump_address);
    result = (result << 26) | masked_jump_address;

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
    // IO Setup
    let input_fn = &args.input_as;
    let output_fn = &args.output_as;

    let output_file: File = match File::create(output_fn) {
        Ok(v) => v,
        Err(_) => return Err("Failed to open output file"),
    };

    // Read input
    let file_contents: String = match fs::read_to_string(input_fn) {
        Ok(v) => v,
        Err(_) => return Err("Failed to read input file contents"),
    };

    // Parse labels (TODO : Make more robust later)
    let mut current_addr: u32 = TEXT_ADDRESS_BASE;
    let mut labels: HashMap<&str, u32> = HashMap::new();
    for line in file_contents.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(label_str) = label(line.trim()) {
            println!("Inserting label {} at {:x}", label_str, current_addr);
            labels.insert(label_str, current_addr);
            continue;
        }

        current_addr += MIPS_INSTR_BYTE_WIDTH;
    }
    current_addr = TEXT_ADDRESS_BASE;

    // Tokenize input
    let mut tokens = tokenize(&file_contents);

    // Assembler setup
    let mut state = AssemblerState::Initial;
    let mut r_struct = R {
        shamt: 0,
        funct: 0,
        form: RForm::None,
    };
    let mut i_struct = I {
        opcode: 0,
        form: IForm::None,
    };
    let mut j_struct = J { opcode: 0 };
    let mut args: Vec<&str> = Vec::new();

    // Iterate over all tokens
    while !tokens.is_empty() {
        let token = tokens.remove(0);

        // Scan tokens in
        match state {
            AssemblerState::Initial => match token {
                "main:" => state = AssemblerState::Scanning,
                _ => return Err("Code must begin with 'main' label"),
            },
            AssemblerState::Scanning => {
                match (
                    label(token),
                    r_operation(token),
                    i_operation(token),
                    j_operation(token),
                ) {
                    (Ok(label_str), _, _, _) => {
                        println!("Found label {} at address {}", label_str, labels[label_str]);
                        continue;
                    }
                    (_, Ok(instr_info), _, _) => {
                        state = AssemblerState::CollectingRArguments;

                        println!("-----------------------------------");
                        println!(
                            "[R] {} - shamt [{:x}] - funct [{:x}]",
                            token, instr_info.shamt, instr_info.funct
                        );

                        r_struct = instr_info;
                        args.push(token)
                    }
                    (_, _, Ok(instr_info), _) => {
                        state = AssemblerState::CollectingIArguments;

                        println!("-----------------------------------");
                        println!("[I] {} - opcode [{:x}]", token, instr_info.opcode);

                        i_struct = instr_info;
                        args.push(token);
                    }
                    (_, _, _, Ok(instr_info)) => {
                        state = AssemblerState::CollectingJArguments;

                        println!("-----------------------------------");
                        println!("[J] {} - opcode [{:x}]", token, instr_info.opcode);

                        j_struct = instr_info;
                        args.push(token);
                    }
                    _ => return Err("Failed to parse mnemonic"),
                }
            }

            AssemblerState::CollectingRArguments
            | AssemblerState::CollectingIArguments
            | AssemblerState::CollectingJArguments => {
                let filtered_token = if token.ends_with(',') {
                    match token.strip_suffix(',') {
                        Some(s) => s,
                        _ => "UNKNOWN_TOKEN",
                    }
                } else {
                    token
                };
                // Filter out comma
                args.push(filtered_token);
            }
        }

        // Try to assemble if args collected
        match state {
            AssemblerState::CollectingRArguments => match assemble_r(&mut r_struct, args.clone()) {
                Ok(assembled_r) => {
                    if write_u32(&output_file, assembled_r).is_err() {
                        return Err("Failed to write to output binary");
                    }

                    state = AssemblerState::Scanning;
                    args.clear();
                }
                Err(_) => continue,
            },
            AssemblerState::CollectingIArguments => {
                match assemble_i(
                    &mut i_struct,
                    args.clone(),
                    &mut labels.clone(),
                    current_addr,
                ) {
                    Ok(assembled_i) => {
                        if write_u32(&output_file, assembled_i).is_err() {
                            return Err("Failed to write to output binary");
                        }

                        state = AssemblerState::Scanning;
                        args.clear();
                    }
                    Err(_) => continue,
                }
            }
            AssemblerState::CollectingJArguments => {
                match assemble_j(&mut j_struct, args.clone(), &mut labels.clone()) {
                    Ok(assembled_j) => {
                        if write_u32(&output_file, assembled_j).is_err() {
                            return Err("Failed to write to output binary");
                        }

                        state = AssemblerState::Scanning;
                        args.clear();
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
            _ => (),
        };

        current_addr += MIPS_INSTR_BYTE_WIDTH;
    }

    if args.is_empty() {
        Ok(())
    } else {
        println!("Args: {:?}", args);
        Err("Unterminated parsing, likely due to uncaught syntax error")
    }
}
