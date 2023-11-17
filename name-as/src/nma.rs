/// NAME Mips Assembler
use crate::args::{self, Args};
use crate::config::Config;
use crate::lineinfo::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str;

fn mask_u8(n: u8, x: u8) -> Result<u8, &'static str> {
    let out = n & ((1 << x) - 1);
    if out != n {
        Err("Masking error")
    } else {
        Ok(out)
    }
}

fn mask_u32(n: u32, x: u8) -> Result<u32, &'static str> {
    let out = n & ((1 << x) - 1);
    if out != n {
        Err("Masking error")
    } else {
        Ok(out)
    }
}

fn base_parse(input: &str) -> Result<u32, &'static str> {
    if let Some(literal) = input.strip_prefix("0x") {
        // Hexadecimal
        u32::from_str_radix(literal, 16).map_err(|_| "Failed to parse as hexadecimal")
    } else if let Some(literal) = input.strip_prefix("0b") {
        // Binary
        u32::from_str_radix(literal, 2).map_err(|_| "Failed to parse as binary")
    } else if input.starts_with('0') && input.len() > 1 {
        // Octal
        u32::from_str_radix(&input[1..], 8).map_err(|_| "Failed to parse as octal")
    } else {
        // Decimal
        input
            .parse::<u32>()
            .map_err(|_| "Failed to parse as decimal")
    }
}

const TEXT_ADDRESS_BASE: u32 = 0x400000;
const _DATA_ADDRESS_BASE: u32 = 0x10000000;
const MIPS_INSTR_BYTE_WIDTH: u32 = 4;

// Controls whether data is being assembled into .text, .data, etc
enum AssemblyMode {
    TextMode,
    DataMode,
}

/// The form of an R-type instruction, specificially
/// which arguments it expects in which order
enum RForm {
    RdRsRt,
    RdRtShamt,
    Rs,
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
        "or" => Ok(R {
            shamt: 0,
            funct: 0x25,
            form: RForm::RdRsRt,
        }),
        "nor" => Ok(R {
            shamt: 0,
            funct: 0x27,
            form: RForm::RdRsRt,
        }),
        "xor" => Ok(R {
            shamt: 0,
            funct: 0x26,
            form: RForm::RdRsRt,
        }),
        "slt" => Ok(R {
            shamt: 0,
            funct: 0x2a,
            form: RForm::RdRsRt,
        }),
        "jr" => Ok(R {
            shamt: 0,
            funct: 0x08,
            form: RForm::Rs,
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
        "slti" => Ok(I {
            opcode: 0xa,
            form: IForm::RtRsImm,
        }),
        "addi" => Ok(I {
            opcode: 0x8,
            form: IForm::RtRsImm,
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

/// Write a u32 into a file, zero-padded to 32 bits (4 bytes)
pub fn write_u32(mut file: &File, data: u32) -> std::io::Result<()> {
    fn convert_endianness(input: u32) -> u32 {
        ((input & 0x000000FF) << 24)
            | ((input & 0x0000FF00) << 8)
            | ((input & 0x00FF0000) >> 8)
            | ((input & 0xFF000000) >> 24)
    }

    const PADDED_LENGTH: usize = 4;

    // Create a 4-length buffer
    let mut padded_buffer: [u8; PADDED_LENGTH] = [0; PADDED_LENGTH];

    // Convert data into bytes
    let bytes: [u8; PADDED_LENGTH] = (convert_endianness(data)).to_be_bytes();

    // Copy bytes into buffer at offset s.t. value is left-padded with 0s
    let copy_index = PADDED_LENGTH - bytes.len();
    padded_buffer[copy_index..].copy_from_slice(&bytes);

    // Write to file
    file.write_all(&padded_buffer)
}

/// Converts a numbered mnemonic ($t0, $s8, etc) or literal (55, 67, etc) to its integer representation
fn reg_number(mnemonic: &str) -> Result<u8, &'static str> {
    if mnemonic.len() != 3 {
        println!("{}", mnemonic);
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

/// Enforce a specific length for a given vector
fn enforce_length(arr: &Vec<&str>, len: usize) -> Result<u32, String> {
    if arr.len() != len {
        Err(format!("Expected {} arguments but got {}", len, arr.len()))
    } else {
        Ok(0)
    }
}

/// Assembles an R-type instruction
fn assemble_r(r_struct: R, r_args: Vec<&str>) -> Result<u32, String> {
    let mut rs: u8;
    let mut rt: u8;
    let mut rd: u8;
    let mut shamt: u8;

    match r_struct.form {
        RForm::RdRsRt => {
            enforce_length(&r_args, 3)?;
            rd = assemble_reg(r_args[0])?;
            rs = assemble_reg(r_args[1])?;
            rt = assemble_reg(r_args[2])?;
            shamt = r_struct.shamt;
        }
        RForm::RdRtShamt => {
            enforce_length(&r_args, 3)?;
            rd = assemble_reg(r_args[0])?;
            rs = 0;
            rt = assemble_reg(r_args[1])?;
            shamt = match base_parse(r_args[2]) {
                Ok(v) => v as u8,
                Err(_) => return Err("Failed to parse shamt".to_string()),
            }
        }
        RForm::Rs => {
            enforce_length(&r_args, 1)?;
            rd = 0;
            rs = assemble_reg(r_args[0])?;
            rt = 0;
            shamt = r_struct.shamt;
        }
    };

    let mut funct = r_struct.funct;

    // Mask
    rs = mask_u8(rs, 5)?;
    rt &= mask_u8(rt, 5)?;
    rd &= mask_u8(rd, 5)?;
    shamt &= mask_u8(shamt, 5)?;
    funct &= mask_u8(funct, 6)?;

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
    i_struct: I,
    i_args: Vec<&str>,
    labels: &HashMap<&str, u32>,
    instr_address: u32,
) -> Result<u32, String> {
    let mut rs: u8;
    let mut rt: u8;
    let imm: u16;

    match i_struct.form {
        IForm::RtImm => {
            enforce_length(&i_args, 2)?;
            rs = 0;
            rt = assemble_reg(i_args[0])?;
            imm = match base_parse(i_args[1]) {
                Ok(v) => v as u16,
                Err(_) => return Err(format!("Failed to parse immediate {}", i_args[1])),
            }
        }
        IForm::RtImmRs => {
            // Immediate can default to 0 if not included in instructions
            // such as 'ld $t0, ($t1)'
            let i_args_catch = if i_args.len() == 2 {
                let mut c = i_args.clone();
                c.insert(1, "0");
                c
            } else {
                i_args.clone()
            };
            enforce_length(&i_args_catch, 3)?;
            rt = assemble_reg(i_args_catch[0])?;
            imm = match base_parse(i_args_catch[1]) {
                Ok(v) => v as u16,
                Err(_) => return Err(format!("Failed to parse immediate {}", i_args[1])),
            };
            rs = assemble_reg(i_args_catch[2])?;
        }
        IForm::RsRtLabel => {
            enforce_length(&i_args, 3)?;
            rs = assemble_reg(i_args[0])?;
            rt = assemble_reg(i_args[1])?;
            match labels.get(i_args[2]) {
                // Subtract byte width due to branch delay
                Some(v) => imm = (((*v) - instr_address - 1) / MIPS_INSTR_BYTE_WIDTH) as u16,
                None => return Err(format!("Undeclared label {}", i_args[2])),
            }
        }
        IForm::RtRsImm => {
            enforce_length(&i_args, 3)?;
            rt = assemble_reg(i_args[0])?;
            rs = assemble_reg(i_args[1])?;
            imm = match base_parse(i_args[2]) {
                Ok(v) => v as u16,
                Err(_) => return Err(format!("Failed to parse immediate {}", i_args[2])),
            };
        }
    };

    let mut opcode = i_struct.opcode;

    // Mask
    println!("Masking rs");
    rs = mask_u8(rs, 5)?;
    println!("Masking rt");
    rt = mask_u8(rt, 5)?;
    println!("Masking opcode");
    opcode = mask_u8(opcode, 6)?;
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
fn assemble_j(j_struct: J, j_args: Vec<&str>, labels: &HashMap<&str, u32>) -> Result<u32, String> {
    enforce_length(&j_args, 1)?;

    let jump_address: u32 = labels[j_args[0]];
    println!("Masking jump address");
    println!("Jump address original: {}", jump_address);
    let mut masked_jump_address = mask_u32(jump_address, 28)?;
    println!("Jump address masked: {}", masked_jump_address);
    if jump_address != masked_jump_address {
        return Err(format!(
            "Tried to assemble illegal jump address {}",
            jump_address
        ));
    }

    // Byte-align jump address
    masked_jump_address >>= 2;

    let mut opcode = j_struct.opcode;

    // Mask
    println!("Masking opcode");
    opcode = mask_u8(opcode, 6)?;
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

pub fn get_pseudo_length(mnemonic: String) -> Result<u32, String> {
    match mnemonic.clone().as_str() {
        "la" | "li" => Ok(2),
        _ => Err(format!("{} is not a pseudo-instruction", mnemonic)),
    }
}

/// Converts a pseudop into its real counterparts
pub fn expand_pseudo<'a>(
    mnemonic: &'a String,
    pseudo_args: Vec<&'a str>,
    labels: &HashMap<&str, u32>,
) -> Result<Vec<(&'a str, Vec<String>)>, String> {
    let result = match mnemonic.clone().as_str() {
        "la" | "li" => {
            let args_catch = if pseudo_args.len() == 2 {
                let mut c = pseudo_args.clone();
                c.insert(1, "0");
                c
            } else {
                pseudo_args.clone()
            };
            enforce_length(&args_catch, 3)?;
            let offset = base_parse(args_catch[1])?;
            let base = match (labels.get(args_catch[2]), base_parse(args_catch[2])) {
                (Some(v), _) => *v,
                (_, Ok(n)) => n,
                (None, Err(_)) => return Err(format!("Bad pseudop argument {}", args_catch[2])),
            };
            let address = offset + base;

            println!("{:x} {:x} {:x}", offset, base, address);
            let (upper, lower) = ((address & 0xFFFF0000) >> 16, address & 0xFFFF);
            println!("{:x} {:x}", upper, lower);

            Ok(vec![
                ("lui", vec!["$at".to_string(), upper.to_string()]),
                (
                    "ori",
                    vec![
                        args_catch[0].to_string(),
                        "$at".to_string(),
                        lower.to_string(),
                    ],
                ),
            ])
        }
        _ => Ok(vec![(
            mnemonic.as_str(),
            pseudo_args.iter().map(|x| x.to_string()).collect(),
        )]),
    };

    if let Ok(v) = &result {
        if v.len() as u32 != get_pseudo_length(mnemonic.to_string()).unwrap_or(v.len() as u32) {
            return Err(format!(
                "Generated pseudo length for {} doesn't match expected length of {}",
                mnemonic,
                get_pseudo_length(mnemonic.to_string())?
            ));
        };
    }

    result
}

use crate::parser::*;
use pest::Parser;

fn scan_macro_args(tokens: &[String]) -> Vec<String> {
    tokens
        .to_vec()
        .iter()
        .map(|token| token.chars().filter(|c| !"(),".contains(*c)).collect())
        .collect()
}

pub fn preprocess(mut input: String, input_as: &str) -> Result<String, String> {
    let mut out = Vec::new();

    let mut eqv: HashMap<String, String> = HashMap::new();
    let mut mac: HashMap<String, (Vec<String>, String)> = HashMap::new();

    let mut collecting_macro = false;
    let mut macro_name: String = "".to_string();
    let mut macro_args: Vec<String> = Vec::new();
    let mut macro_buf = String::new();

    let input_path = PathBuf::from(input_as);
    let base_folder = input_path.parent().unwrap();

    let regex = Regex::new(".include \"(?P<include_fn>.*)\"").unwrap();
    for (_, [include_fn]) in regex.captures_iter(&input.clone()).map(|c| c.extract()) {
        let file_contents: String = match fs::read_to_string(base_folder.join(include_fn)) {
            Ok(v) => v,
            Err(_) => return Err("Failed to read input file contents".to_string()),
        };
        input = regex.replace_all(&input, file_contents).to_string();
    }

    for mut line in input.lines() {
        line = line.trim();

        // Filter comments
        if let Some(pos) = line.find('#') {
            line = &line[0..pos]; // Remove the comment part
        }

        if line.is_empty() {
            continue;
        }

        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        // Handle .eqv directives
        if tokens[0] == ".eqv" {
            eqv.insert(tokens[1].clone(), tokens[2].clone());
            continue;
        }

        // Handle .macro directives
        if tokens[0] == ".macro" {
            collecting_macro = true;
            macro_name = tokens[1].clone();
            // Remove the '(', ')', and ',',
            macro_args = scan_macro_args(&tokens[2..]);
            continue;
        } else if tokens[0] == ".end_macro" {
            collecting_macro = false;
            mac.insert(macro_name.clone(), (macro_args.clone(), macro_buf.clone()));
            continue;
        }

        // Replace via eqv
        let mut tokens: Vec<String> = tokens
            .iter()
            .map(|token| {
                let mut subbed_token = token.to_string();
                for eqv_key in eqv.keys() {
                    subbed_token = subbed_token.replace(eqv_key, eqv.get(eqv_key).unwrap())
                }
                subbed_token.to_owned()
            })
            .collect::<Vec<String>>();

        // Replace via macro
        if let Some(scanned_macro) = mac.get(tokens[0].as_str()) {
            let input_args: Vec<String> = scan_macro_args(&tokens[1..]);
            tokens.clear();
            for macro_line in scanned_macro.1.lines() {
                let mut subbed_in_line = macro_line.to_string().clone();
                for (index, arg) in scanned_macro.0.iter().enumerate() {
                    subbed_in_line = subbed_in_line.replace(arg, &input_args[index]);
                }
                out.push(subbed_in_line);
            }
            continue;
        }

        if collecting_macro {
            macro_buf += (tokens.join(" ") + "\n").as_str();
        } else {
            out.push(tokens.join(" "));
        }
    }

    Ok(out.join("\n"))
}

// General assembler entrypoint
pub fn assemble(program_config: &Config, program_arguments: Args) -> Result<(), String> {
    // IO Setup
    let input_fn = &program_arguments.input_as;
    let output_fn = &program_arguments.output_as;

    let output_file: File = match File::create(output_fn) {
        Ok(v) => v,
        Err(_) => return Err("Failed to open output file".to_string()),
    };

    // Read input
    let mut file_contents: String = match fs::read_to_string(input_fn) {
        Ok(v) => v,
        Err(_) => return Err("Failed to read input file contents".to_string()),
    };

    // Preprocess
    file_contents = if program_config.preprocess {
        match preprocess(file_contents, input_fn) {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        }
    } else {
        file_contents
    };
    println!("{}", file_contents);

    // Parse into CST
    let cst = parse_rule(
        MipsParser::parse(Rule::vernacular, file_contents.as_str())
            .expect("Failed to parse")
            .next()
            .unwrap(),
    );

    // Set up line info
    let lineinfo_fn = format!("{}.li", &program_arguments.output_as);
    let mut lineinfo: Vec<LineInfo> = vec![];

    let vernac_sequence: Vec<MipsCST> = if let MipsCST::Sequence(v) = cst {
        v
    } else {
        vec![cst]
    };

    // General setup
    let mut _assembly_mode = AssemblyMode::TextMode;
    let mut text_section_address: u32 = TEXT_ADDRESS_BASE;

    // Assign addresses to labels
    let mut labels: HashMap<&str, u32> = HashMap::new();
    for sub_cst in &vernac_sequence {
        // The instruction width of the currently-parsed instruction
        // Should be 1 if not a pseudo
        let unwrapped_pseudo_length = match sub_cst {
            MipsCST::Label(label_str) => {
                println!(
                    "Inserting label {} at 0x{:x}",
                    label_str, text_section_address
                );
                labels.insert(label_str, text_section_address);
                continue;
            }
            MipsCST::Sequence(_) => unreachable!(),
            MipsCST::Directive(_, _) => continue,
            MipsCST::Instruction(mnemonic, _) => {
                get_pseudo_length(mnemonic.to_string()).unwrap_or(1)
            }
        };

        text_section_address += MIPS_INSTR_BYTE_WIDTH * unwrapped_pseudo_length;
    }

    text_section_address = TEXT_ADDRESS_BASE;

    // Assemble instructions
    for sub_cst in vernac_sequence {
        match sub_cst {
            MipsCST::Instruction(root_mnemonic, root_args) => {
                let rm = root_mnemonic.to_string();
                let to_assemble = expand_pseudo(&rm, root_args.clone(), &labels)?;

                // Update line info
                lineinfo.push(LineInfo {
                    instr_addr: text_section_address,
                    line_number: 0,
                    line_contents: instr_to_str(root_mnemonic, &root_args),
                    psuedo_op: "".to_string(),
                });

                for (mnemonic, args) in to_assemble.iter() {
                    let args = args.iter().map(|x| x.as_str()).collect();
                    if let Ok(instr_info) = r_operation(mnemonic) {
                        println!("-----------------------------------");
                        println!(
                            "[R] {} - shamt [{:x}] - funct [{:x}] - args {:?}",
                            mnemonic, instr_info.shamt, instr_info.funct, args
                        );
                        match assemble_r(instr_info, args) {
                            Ok(assembled_r) => {
                                if write_u32(&output_file, assembled_r).is_err() {
                                    return Err("Failed to write to output binary".to_string());
                                }
                            }
                            Err(e) => return Err(e.to_string()),
                        }
                    } else if let Ok(instr_info) = i_operation(mnemonic) {
                        println!("-----------------------------------");
                        println!(
                            "[I] {} - opcode [{:x}] - args {:?}",
                            mnemonic, instr_info.opcode, args
                        );

                        match assemble_i(instr_info, args, &labels, text_section_address) {
                            Ok(assembled_i) => {
                                if write_u32(&output_file, assembled_i).is_err() {
                                    return Err("Failed to write to output binary".to_string());
                                }
                            }
                            Err(e) => return Err(e.to_string()),
                        }
                    } else if let Ok(instr_info) = j_operation(mnemonic) {
                        println!("-----------------------------------");
                        println!(
                            "[J] {} - opcode [{:x}] - args {:?}",
                            mnemonic, instr_info.opcode, args
                        );

                        match assemble_j(instr_info, args, &labels) {
                            Ok(assembled_j) => {
                                if write_u32(&output_file, assembled_j).is_err() {
                                    return Err("Failed to write to output binary".to_string());
                                }
                            }
                            Err(e) => return Err(e.to_string()),
                        }
                    } else {
                        return Err("Failed to match instruction".to_string());
                    }

                    text_section_address += MIPS_INSTR_BYTE_WIDTH;
                }
            }
            MipsCST::Directive(mnemonic, _args) => match mnemonic {
                "text" => _assembly_mode = AssemblyMode::TextMode,
                "data" => _assembly_mode = AssemblyMode::DataMode,
                "eqv" | "macro" => (),
                _ => return Err(format!("Directive .{} not yet supported", mnemonic).to_string()),
            },
            _ => continue,
        };
    }

    if program_arguments.line_info {
        if let Err(e) = lineinfo_export(lineinfo_fn, lineinfo) {
            return Err(e.to_string());
        }
    }

    Ok(())
}
