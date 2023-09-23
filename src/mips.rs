use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::Cursor;

pub const DOT_TEXT: u32 = 0x00400000;
const DOT_TEXT_MAX_LENGTH: u32 = 0x1000;
const LEN_TEXT_INITIAL: usize = 200;

pub(crate) struct Mips {
    pub regs: [u32; 32],
    pub floats: [f32; 32],
    pub mult_hi: u32,
    pub mult_lo: u32,
    pub pc: usize,

    // A list of vectors of memory pools, their base addresses, and their
    // maximum lengths.
    pub memories: Vec<(Vec<u8>, u32, u32)>
}

#[derive(Debug)]
#[derive(PartialEq, Copy, Clone)]
pub(crate) enum ExecutionErrors {
    // The program attempted to access an address that was within a
    // valid range, but was outside the current allocation for that range.
    // This should be treated as a warning, and read out as zero.
    MemoryObviouslyUninitializedAccess,
    // The program attempted to read from an area for which no valid range existed.
    MemoryIllegalAccess,

    // The program is done executing.
    ProgramComplete,

    UndefinedInstruction
}

impl fmt::Display for ExecutionErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}


impl Default for Mips {
    fn default() -> Self {
        Self {
            regs: [0; 32],
            floats: [0f32; 32],
            mult_hi: 0,
            mult_lo: 0,
            pc: 0x00400000,
            memories: vec![
                (Vec::with_capacity(LEN_TEXT_INITIAL), DOT_TEXT, DOT_TEXT_MAX_LENGTH)   
            ]
        }
    }
}

struct Rtype {
    rs: usize,
    rt: usize,
    rd: usize,
    shamt: u8,
    funct: u8
}

struct Itype {
    opcode: u32,
    rs: usize,
    rt: usize,
    imm: u16
}

// struct Jtype
// struct Ftype

enum Instructions {
    R(Rtype),
    I(Itype),
    //J and F type
}

impl Mips {

    fn dispatch_r(&mut self, ins: Rtype) -> Result<(), ExecutionErrors> {
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
                self.regs[ins.rd] = self.regs[ins.rt] + self.regs[ins.rs];
                //Todo- catch overflows
            }
            // Subtract
            0x22 => {
                //Todo- catch overflows
                self.regs[ins.rd] = self.regs[ins.rt] + self.regs[ins.rs];
            }
            // Xor
            0x26 => {
                self.regs[ins.rd] = self.regs[ins.rt] ^ self.regs[ins.rs];
            }
            _ => return Err(ExecutionErrors::UndefinedInstruction)
        }
        Ok(())
    }
    fn dispatch_i(&mut self, ins: Itype) -> Result<(), ExecutionErrors> {

        let memory_address = (ins.rt as i64 + (ins.imm as i64)) as u32;

        match ins.opcode {
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

            _ => return Err(ExecutionErrors::UndefinedInstruction)
        }
        Ok(())
    }

    fn decode(&self, instruction: u32) -> Instructions {
        let opcode = instruction >> 26 & 0b11111;
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
            // 0x2 | 0x3 => {
            //     Instructions::Jtype(Jtype {

            //     })
            // }
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

    // This function attempts to access memory and returns an error if that memory doesn't exist
    pub fn read_b(&mut self, address: u32) -> Result<u8, ExecutionErrors> {
        if let Some((memory, offset)) = self.map_memory(address) {
            if let Some(value) = memory.get(offset as usize) {
                Ok(*value)
            }
            else {
                Err(ExecutionErrors::MemoryObviouslyUninitializedAccess)
            }
        }
        else { Err(ExecutionErrors::MemoryIllegalAccess) }
    }
    pub fn read_h(&mut self, address: u32) -> Result<u16, ExecutionErrors> {
        let bytes = [self.read_b(address)?, self.read_b(address + 1)?];
        Ok(Cursor::new(bytes).read_u16::<LittleEndian>().unwrap())
    }
    pub fn read_w(&mut self, address: u32) -> Result<u32, ExecutionErrors> {
        let bytes = [self.read_b(address)?, self.read_b(address + 1)?,
                        self.read_b(address + 2)?, self.read_b(address + 3)?];
        Ok(Cursor::new(bytes).read_u32::<LittleEndian>().unwrap())
    }

    
    // I'm clueless on how to expand this. I need to talk to Cole
    pub fn write_b(&mut self, address: u32, value: u8) -> Result<(), ExecutionErrors> {
        if let Some((memory, offset)) = self.map_memory(address) {
            memory[offset as usize] = value;
            Ok(())
        }
        else { Err(ExecutionErrors::MemoryIllegalAccess) }
    }
    pub fn write_h(&mut self, address: u32, value: u16) -> Result<(), ExecutionErrors> {
        let mut bytes = vec![];
        bytes.write_u16::<LittleEndian>(value).unwrap();
        self.write_b(address, bytes[0])?;
        self.write_b(address, bytes[1])?;
        Ok(())
    }
    pub fn write_w(&mut self, address: u32, value: u32) -> Result<(), ExecutionErrors> {
        let mut bytes = vec![];
        bytes.write_u32::<LittleEndian>(value).unwrap();
        self.write_b(address, bytes[0])?;
        self.write_b(address, bytes[1])?;
        self.write_b(address, bytes[2])?;
        self.write_b(address, bytes[3])?;
        Ok(())
    }

    pub fn step_one(&mut self) -> Result<(), ExecutionErrors> {
        let opcode = self.read_w(self.pc as u32)?;
        self.pc += 1;
        let instruction = self.decode(opcode);

        match instruction {
            Instructions::R(rtype) => self.dispatch_r(rtype),
            Instructions::I(itype) => self.dispatch_i(itype)
        }
    }
}