// NAME Mips Assembler

use std::fs;
use std::str;
use crate::u5::U5;
use crate::u6::U6;

const R_EXPECTED_ARGS : usize = 3;

pub struct R {
    shamt: U5,
    funct: U6
}

const I_EXPECTED_ARGS : usize = 2;

pub struct I {
    opcode: U5
}

pub fn r_operation(mnemonic: &str) -> Result<R, &'static str> {
    match mnemonic {
        "add" => Ok(R { shamt : U5 {value : 0}, funct : U6 {value : 0x20}}),
        "sub" => Ok(R { shamt : U5 {value : 0}, funct : U6 {value : 0x22}}),
        "sll" => Ok(R { shamt : U5 {value : 0}, funct : U6 {value : 0x00}}),
        "srl" => Ok(R { shamt : U5 {value : 0}, funct : U6 {value : 0x02}}),
        _ => Err("Failed to match R-instr mnemonic"),
    }
}

pub fn i_operation(mnemonic: &str) -> Result<I, &'static str> {
    match mnemonic {
        "ori" => Ok(I { opcode: U5 {value : 0xd} }),
        _ => Err("Failed to match I-instr mnemonic"),
    }
}

// pub fn is_R(mnemonic: &str) -> bool {
//     match mnemonic {
//         | "add"   | "addu" | "and"
//         | "jr"    | "nor"  | "or"
//         | "slt"   | "sltu" | "sll"
//         | "srl"   | "sub"  | "subu"
//         | "div"   | "divu" | "mfhi"
//         | "mflo"  | "mfc0" | "mult"
//         | "multu" | "sra"
//         => true,
//         _ => false
//     }
// }

pub fn read_file(file_path: &str) -> String {
    fs::read_to_string(file_path)
        .expect("Failed to read the file")
}

pub fn tokenize(raw_text: &str) -> Vec<&str> {
    raw_text
        .split_whitespace()
        .collect::<Vec<&str>>()
}

#[derive(Debug)]
enum AssemblerState {
    Initial,
    CollectingRArguments,
    CollectingIArguments
}

fn assemble_r(r_struct : &mut R, r_args : Vec<&str>) -> u32 {
    // opcode : 31 - 26
    let mut result = 0x00000;
    // rs
    // rt
    // rd

    // shamt : 6 - 10
    let shamt : u32 = r_struct.shamt.into();
    result = (result << 5) | shamt;

    // funct : 0 - 5
    let funct : u32 = r_struct.funct.into();
    result = (result << 6) | funct;

    println!("{:0width$b}", result, width=32);
    result
}

pub fn assemble(file_path : &str) -> () {
    let file_contents = read_file(file_path);
    let tokens = tokenize(&file_contents);

    let mut state = AssemblerState::Initial;
    let mut r_struct: R = R { shamt : U5 {value : 0}, funct : U6 {value : 0}};
    let mut r_args: Vec<&str> = Vec::new();
    let mut i_struct: I = I { opcode : U5 {value : 0}};
    let mut i_args: Vec<&str> = Vec::new();

    for token in tokens {
        // println!("State: {:?}", state);
        match state {
            AssemblerState::Initial => {
                // two options here -
                // 1. nest the (match r .. else match i .. else ...)
                // 2. add is_r, is_i etc
                // 3. Put them one after another like this (bad, will remove)
                // leaning towards 1
                match r_operation(&token) {
                    Ok(instr_info) => {
                        state = AssemblerState::CollectingRArguments;

                        println!("[R] {} - shamt [{:x}] - funct [{:x}]", token, instr_info.shamt, instr_info.funct);
                        
                        r_struct = instr_info;
                        r_args.clear();
                        r_args.push(token) 
                    },
                    _ => match i_operation(&token) {
                            Ok(instr_info) => {
                                state = AssemblerState::CollectingIArguments;

                                println!("[I] {} - opcode [{:x}]", token, instr_info.opcode);

                                i_struct = instr_info;
                                i_args.clear();
                                i_args.push(token)
                            },
                            _ => ()
                        }
                }
            },
            AssemblerState::CollectingRArguments => {
                // println!("Collecting args {:?}", r_args);
                if r_args.len() == R_EXPECTED_ARGS {
                    let _assembled_r = assemble_r(&mut r_struct, r_args.clone());
                    state = AssemblerState::Initial;
                } else {
                    r_args.push(token);
                }
            },
            _ => ()
        }
    }
}
