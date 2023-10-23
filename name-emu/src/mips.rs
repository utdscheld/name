use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use std::fs::File;
use std::io::Write;

use crate::exception::{ExecutionErrors, ExecutionEvents};

pub const DOT_TEXT_START_ADDRESS: u32 = 0x00400000;
const DOT_TEXT_MAX_LENGTH: u32 = 0x1000;
const LEN_TEXT_INITIAL: usize = 200;
const MIPS_INSTRUCTION_LENGTH: usize = 4;

pub const REGISTER_NAMES: [&str; 32] = [
    "$zero",
    "$at",
    "$v0",
    "$v1",
    "$a0",
    "$a1",
    "$a2",
    "$a3",
    "$t0",
    "$t1",
    "$t2",
    "$t3",
    "$t4",
    "$t5",
    "$t6",
    "$t7",
    "$s0",
    "$s1",
    "$s2",
    "$s3",
    "$s4",
    "$s5",
    "$s6",
    "$s7",
    "$t8",
    "$t9",
    "$k0",
    "$k1",
    "$gp",
    "$sp",
    "$fp",
    "$ra"
];
pub const PC_NAME: &str = "$pc";

#[derive(Debug)]
enum BranchDelays {
    NotActive,
    Set,
    Ready
}

#[derive(Debug)]
pub(crate) struct Mips {
    pub regs: [u32; 32],
    pub floats: [f32; 32],
    pub mult_hi: u32,
    pub mult_lo: u32,
    pub pc: usize,

    // Branch delay slots are implemented by filling this buffer with the
    // branch target, which will be triggered after the following instruction
    branch_delay_target: u32,
    branch_delay_status: BranchDelays,
    

    // A list of vectors of memory pools, their base addresses, and their
    // lengths.
    // Memories allocated at runtime will actually have Vec lengths shorter
    // than this by 0x10. This is intended to alert the user that they 
    // probably wrote out of bounds, allowing us to return a clearer exception
    // and explanation as to what happened.
    pub memories: Vec<(Vec<u8>, u32, u32)>,
    // The end of the MIPS program. In NAME, the program terminates when no more instructions exist
    // (as in, falling off the bottom is valid).
    pub stop_address: usize,
    
    // Memory for the result of a previous instruction (useful for tracking exceptions)
    pub prev_ins_result: Result<(), ExecutionErrors>
}


impl Default for Mips {
    fn default() -> Self {
        Self {
            regs: [0; 32],
            floats: [0f32; 32],
            mult_hi: 0,
            mult_lo: 0,
            pc: DOT_TEXT_START_ADDRESS as usize,
            branch_delay_target: 0,
            branch_delay_status: BranchDelays::NotActive,
            memories: vec![
                (vec![0; LEN_TEXT_INITIAL], DOT_TEXT_START_ADDRESS, DOT_TEXT_MAX_LENGTH)   
            ],
            stop_address: DOT_TEXT_START_ADDRESS as usize,
            prev_ins_result: Ok(())
        }
    }
}

#[derive(Debug)]
struct Rtype {
    rs: usize,
    rt: usize,
    rd: usize,
    shamt: u8,
    funct: u8
}

#[derive(Debug)]
struct Itype {
    opcode: u32,
    rs: usize,
    rt: usize,
    imm: u16
}

#[derive(Debug)]
struct Jtype {
    opcode: u32,
    dest: u32
}

// struct Jtype
// struct Ftype

#[derive(Debug)]
enum Instructions {
    R(Rtype),
    I(Itype),
    J(Jtype)
    //F type
}

impl Mips {

    fn dispatch_r(&mut self, ins: Rtype, opcode: u32) -> Result<(), ExecutionErrors> {

        match ins.funct {
            // Shift-left logical
            0x0 => {
                self.regs[ins.rd] = self.regs[ins.rt] << ins.shamt;
            }
            // Shift-right logical
            0x2 => {
                self.regs[ins.rd] = self.regs[ins.rt] >> ins.shamt;
            }
            // Add
            0x20 => {
                let result = self.regs[ins.rt].checked_add(self.regs[ins.rs]);
                match result {
                    Some(value) => {self.regs[ins.rd] = value;}
                    None => {
                        return Err(ExecutionErrors::IntegerOverflow { 
                        rt: ins.rt, 
                        rs: ins.rs, 
                        value1: self.regs[ins.rt],
                        value2: self.regs[ins.rs]
                        });
                    }
                }
            }
            // Subtract
            0x22 => {
                let result = self.regs[ins.rt].checked_sub(self.regs[ins.rs]);
                match result {
                    Some(value) => {self.regs[ins.rd] = value;}
                    None => {
                        return Err(ExecutionErrors::IntegerOverflow { 
                        rt: ins.rt, 
                        rs: ins.rs, 
                        value1: self.regs[ins.rt],
                        value2: self.regs[ins.rs]
                        });
                    }
                }
            }
            // Or
            0x25 => {
                self.regs[ins.rd] = self.regs[ins.rt] | self.regs[ins.rs];
            }
            // Xor
            0x26 => {
                self.regs[ins.rd] = self.regs[ins.rt] ^ self.regs[ins.rs];
            }
            // Nor
            0x27 => {
                self.regs[ins.rd] = !(self.regs[ins.rt] | self.regs[ins.rs]);
            }
            // Set Less Than
            0x2A => {
                if self.regs[ins.rt] < self.regs[ins.rs] {
                    self.regs[ins.rd] = 1;
                }
                else {
                    self.regs[ins.rd] = 0;
                }
            }
            _ => return Err(ExecutionErrors::UndefinedInstruction {instruction: opcode})
        }
        Ok(())
    }
    fn dispatch_i(&mut self, ins: Itype, opcode: u32) -> Result<(), ExecutionErrors> {

        let memory_address = (ins.rt as i64 + (ins.imm as i64)) as u32;

        match ins.opcode {
            // Set Less Than Immediate (signed)
            // Forcing sign extend through a series of casts. See load byte comments
            0xA => {
                if self.regs[ins.rs] < ins.imm as i8 as i32 as u32 {
                    self.regs[ins.rt] = 1;
                }
                else {
                    self.regs[ins.rt] = 0;
                }
            }
            // // Set Less Than 
            // 0xB => {
            //     if 
            // }
            // Or Immediate
            0xD => {
                // Rust zero-extends unsigned values when up-casting
                self.regs[ins.rt] = self.regs[ins.rs] | ins.imm as u32;
            }
            // Load Upper Immediate
            0xF => {
                self.regs[ins.rt] = (ins.imm as u32) << 16;
            }
            // Load word (0x23) and Load Linked (0x30).
            // A word on Load Linked-- This is an instruction for atomic accesses
            // across SMP processors. NAME does not implement SMP, so this is equal to
            // Load word.
            0x23 | 0x30 =>{
                self.regs[ins.rt] = self.read_w(memory_address)?;
            }
            // Load byte unsigned
            // Note that "as u32" WILL zero extend
            0x24 =>{
                self.regs[ins.rt] = self.read_b(memory_address)? as u32;
            }
            // Load halfword unsigned
            // Note that "as u32" WILL zero extend
            0x25 => {
                self.regs[ins.rt] = self.read_h(memory_address)? as u32;
            }
            // Load byte (signed)
            // Note that I force a sign extension through a convuluted series of casts
            // u8 -> i8 (same bits) -> i32 (more bits, sign extension) -> u32 (same bits)
            0x20 => {
                self.regs[ins.rt] = self.read_b(memory_address)? as i8 as i32 as u32;
            }
            // Load halfword (signed), same deal
            0x21 => {
                self.regs[ins.rt] = self.read_h(memory_address)? as i16 as i32 as u32;
            }
            // Store byte
            0x28 => {
                self.write_b(memory_address, self.regs[ins.rt] as u8)?;
            }
            // Store halfword
            0x29 => {
                self.write_h(memory_address, self.regs[ins.rt] as u16)?;
            }
            // Store word (0x2b) and Store Conditional (0x38).
            // Store Conditional is the second half of Load Linked, and it's an equivalent
            // op for the same reason.
            0x2b | 0x38 => {
                self.write_w(memory_address, self.regs[ins.rt])?;
            }
            // Branch if Equal
            0x4 => {
                if self.regs[ins.rt] == self.regs[ins.rs] {
                    self.branch_delay_target = (ins.imm as u32) << 2;
                    self.branch_delay_status = BranchDelays::Set;
                }
            }
            // Branch if Not Equal
            0x5 => {
                if self.regs[ins.rt] != self.regs[ins.rs] {
                    self.branch_delay_target = (ins.imm as u32) << 2;
                    self.branch_delay_status = BranchDelays::Set;
                }
            }
            

            _ => return Err(ExecutionErrors::UndefinedInstruction {instruction: opcode})
        }
        Ok(())
    }
    fn dispatch_j(&mut self, ins: Jtype, opcode: u32) -> Result<(), ExecutionErrors> {
        // This instruction type takes the top nybble of PC and combines it with
        // a 28-bit range (26 bits as encoded shifted left twice.)
        // Thus, you can jump to anywhere in a 256MB address space.
        match ins.opcode {
            // Jump absolute
            2 => {
                self.branch_delay_status = BranchDelays::Set;
                self.branch_delay_target = self.pc as u32 & 0xF0000000 | (ins.dest << 2);
            }
            // Jump And Link
            3 => {
                self.branch_delay_status = BranchDelays::Set;
                self.branch_delay_target = self.pc as u32 & 0xF0000000 | (ins.dest << 2);
                // $ra = register 31
                self.regs[31] = self.pc as u32 + 8;
            }
            _ => return Err(ExecutionErrors::UndefinedInstruction {instruction: opcode})
        }

        Ok(())
    }

    fn decode(&self, instruction: u32) -> Instructions {
        let opcode = instruction >> 26 & 0b111111;
        match opcode {
            // R-type
            0 => {
                Instructions::R(Rtype {
                    // These are all five-bit fields
                    rs: (instruction >> 21 & 0b11111) as usize,
                    rd: (instruction >> 16 & 0b11111) as usize,
                    rt: (instruction >> 11 & 0b11111) as usize,
                    shamt: (instruction >> 6 & 0b11111) as u8,
                    // This is a six-bit field
                    funct: (instruction & 0b111111) as u8
                })
            }
            // J-type
            0x2 | 0x3 => {
                Instructions::J(Jtype {
                    opcode,
                    // Lower 26 bits of the instruction
                    dest: instruction & 0b11111111111111111111111111
                })
            }
            // I-type
            _ => {
                Instructions::I(Itype {
                    opcode,
                    rs: (instruction >> 21 & 0b11111) as usize,
                    rt: (instruction >> 16 & 0b11111) as usize,
                    imm: instruction as u16
                })
            }
        }
    }

    // Given an address, return a pool of actual memory and the offset with
    // which to access the requested data within it. Note that the offset 
    // address is not necessarily allocated within the returned Vec, 
    // this function just checks ranges.
    fn map_memory(&mut self, address: u32) -> Option<(&mut Vec<u8>, u32)> {
        // Access by the various pools of memory that exist.
        // Note that if an address is supposedly within a region,
        // but that region hasn't been initialized, it won't be within
        // the Vecs size and therefore won't be addressed.
        for (pool, base_address, max_length) in &mut self.memories {
            if (*base_address .. *base_address + *max_length).contains(&address) {
                return Some((pool, address - *base_address))
            }
        }
        None
    }

    // This function attempts to access a byte of memory and returns an error if that memory doesn't exist
    pub fn read_b(&mut self, address: u32) -> Result<u8, ExecutionErrors> {
        if let Some((memory, offset)) = self.map_memory(address) {
            if let Some(value) = memory.get(offset as usize) {
                Ok(*value)
            }
            // Although this memory access was technically within this range,
            // the Vec did not actually fit within it. This means that the user
            // wrote out of bounds of the buffer
            else {
                Err(ExecutionErrors::MemoryObviousOverrunAccess { load_address: address } )
            }
        }
        else { Err(ExecutionErrors::MemoryIllegalAccess { load_address: address } ) }
    }
    // Reads two bytes and returns a halfword
    pub fn read_h(&mut self, address: u32) -> Result<u16, ExecutionErrors> {
        let bytes = [self.read_b(address)?, self.read_b(address + 1)?];
        Ok(Cursor::new(bytes).read_u16::<LittleEndian>().unwrap())
    }
    // Reads four bytes and returns a word
    pub fn read_w(&mut self, address: u32) -> Result<u32, ExecutionErrors> {
        let bytes = [self.read_b(address)?, self.read_b(address + 1)?,
                        self.read_b(address + 2)?, self.read_b(address + 3)?];
        Ok(Cursor::new(bytes).read_u32::<LittleEndian>().unwrap())
    }

    
    // Writes one byte
    pub fn write_b(&mut self, address: u32, value: u8) -> Result<(), ExecutionErrors> {
        if let Some((memory, offset)) = self.map_memory(address) {
            if let Some(element) = memory.get_mut(offset as usize) {
                *element = value;
                Ok(())
            }
            else {
                Err(ExecutionErrors::MemoryObviousOverrunAccess { load_address: address }
                )
            }
        }
        else { Err(ExecutionErrors::MemoryIllegalAccess { load_address: address } ) }
    }
    // Writes a halfword in little endian form
    pub fn write_h(&mut self, address: u32, value: u16) -> Result<(), ExecutionErrors> {
        let mut bytes = vec![];
        bytes.write_u16::<LittleEndian>(value).unwrap();
        self.write_b(address, bytes[0])?;
        self.write_b(address, bytes[1])?;
        Ok(())
    }
    // Writes a word in little endian form
    pub fn write_w(&mut self, address: u32, value: u32) -> Result<(), ExecutionErrors> {
        let mut bytes = vec![];
        bytes.write_u32::<LittleEndian>(value).unwrap();
        self.write_b(address, bytes[0])?;
        self.write_b(address, bytes[1])?;
        self.write_b(address, bytes[2])?;
        self.write_b(address, bytes[3])?;
        Ok(())
    }

    pub fn step_one(&mut self, mut f :&mut File) -> Result<(), ExecutionErrors> {
        let opcode = self.read_w(self.pc as u32)?;
        self.pc += MIPS_INSTRUCTION_LENGTH;

        if self.pc == self.stop_address {
            return Err(ExecutionErrors::Event { event: ExecutionEvents::ProgramComplete });
        }

        let instruction = self.decode(opcode);
        writeln!(f,"{:?}", instruction);

        let ins_result = match instruction {
            Instructions::R(rtype) => self.dispatch_r(rtype, opcode),
            Instructions::I(itype) => self.dispatch_i(itype, opcode),
            Instructions::J(jtype) => self.dispatch_j(jtype, opcode)
        };

        // The zero register is ALWAYS 0.
        // If an instruction wrote to the zero register, discard that result here.
        self.regs[0] = 0;

        if ins_result.is_err() {
            self.pc -= MIPS_INSTRUCTION_LENGTH; // 
        }

        // Branch delay slots are handled here. On the instruction the branch is set,
        // it is not triggered, and instead the state shifts such that after the end of
        // the next instruction the control flow transfer is triggered.
        match self.branch_delay_status {
            BranchDelays::NotActive => (),
            BranchDelays::Set => self.branch_delay_status = BranchDelays::Ready,
            BranchDelays::Ready => {
                self.pc = self.branch_delay_target as usize;
                self.branch_delay_status = BranchDelays::NotActive;
            }
        }

        self.prev_ins_result = ins_result;

        ins_result
    }
}