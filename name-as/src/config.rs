extern crate serde;
extern crate toml;
use serde::Deserialize;

use crate::args::Args;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub config_name: String,
    pub as_cmd: Vec<String>,
}

pub fn backup_config() -> Config {
    Config {
        config_name: "backup config".to_string(),
        as_cmd: ["".to_string()].to_vec(),
    }
}

pub fn parse_config(args: &Args) -> Result<Config, Box<dyn std::error::Error>> {
    let mut toml_content = fs::read_to_string(&args.config_fn)?;

    // Replace patterns
    toml_content = toml_content.replace("{INPUT_AS}", &args.input_as);
    toml_content = toml_content.replace("{OUTPUT_AS}", &args.output_as);

    let config: Config = toml::from_str(&toml_content)?;

    Ok(config)
}
