// We should figure out how to share this file across name-as and name-emu.
// But it raises architectural questions about what this means for portability.
// Are we losing the ability to use other assemblers by doing this?

extern crate serde;
extern crate toml;
use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct LineInfo {
    pub instr_addr: u32,
    pub line_number: u32,
    pub line_contents: String,
    pub psuedo_op: String,
}

#[derive(Deserialize)]
struct LineInfoFile {
    pub lineinfo: Vec<LineInfo>,
}

pub fn lineinfo_import(
    file_contents: String
) -> Result<HashMap<u32, LineInfo>, Box<dyn std::error::Error>> {
    let line_info: LineInfoFile = toml::from_str(&file_contents)?;

    // Code smell— we have to iterate over this entire list and transform it into a HashMap.
    // This is because TOML can't serialize HashMaps with anything other than strings as keys. 
    // So we just serialize as Vec and deserialize as Vec then convert. This is tech debt.
    let out = line_info.lineinfo.into_iter().map(|line| (line.instr_addr, line)).collect::<HashMap<_,_>>();
    
    Ok(out)
}
