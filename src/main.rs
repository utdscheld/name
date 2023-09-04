mod nma {
    pub mod nma;
}

pub mod u5;
pub mod u6;

pub mod args;
pub mod config;

use args::parse_args;
use config::parse_config;
use nma::nma::assemble;

fn main() -> Result<(), &'static str> {
    let cmd_args = parse_args()?;
    let config = parse_config(&cmd_args).expect("Failed to parse configuration file");

    if config.as_cmd.is_empty() {
        assemble(&cmd_args)?;
    } else {
        println!("Config Name:   {}", config.config_name);
        println!("Assembler CMD: {}", config.as_cmd);
    }

    Ok(())
}
