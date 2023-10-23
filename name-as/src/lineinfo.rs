extern crate serde;
extern crate toml;
use serde::Serialize;

use std::fs;

#[derive(Debug, Serialize)]
pub struct LineInfo {
    pub instr_addr: u32,
    pub line_number: u32,
    pub line_contents: String,
    pub psuedo_op: String,
}

#[derive(Serialize)]
struct LineInfoFile {
    pub lineinfo: Vec<LineInfo>,
}

pub fn lineinfo_export(
    filename: String,
    li: Vec<LineInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let toml_data = toml::to_string(&LineInfoFile { lineinfo: li })?;

    fs::write(filename, toml_data)?;

    Ok(())
}
