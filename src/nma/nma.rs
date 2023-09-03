// NAME Mips Assembler

use std::fs;
use std::str;
use crate::u5::U5;
use crate::u6::U6;

pub struct R {
    shamt: U5,
    funct: U6
}

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

pub fn assemble(file_path : &str) -> () {
    let file_contents = read_file(file_path);
    let tokens = tokenize(&file_contents);
    for token in tokens {
        // two options here -
        // 1. nest the (match r .. else match i .. else ...)
        // 2. add is_r, is_i etc
        // 3. Put them one after another like this (bad, will remove)
        // leaning towards 1
        match r_operation(&token) {
            Ok(instr_info) => println!("[R] {} - shamt [{:x}] - funct [{:x}]", token, instr_info.shamt, instr_info.funct),
            _ => ()
        };
        match i_operation(&token) {
            Ok(instr_info) => println!("[I] {} - opcode [{:x}]", token, instr_info.opcode),
            _ => ()
        }
    }
}
