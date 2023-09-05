mod nma {
    pub mod nma;
}

pub mod u5;
pub mod u6;

pub mod args;
pub mod config;

use args::parse_args;
use nma::nma::assemble;

fn main() -> Result<(), &'static str> {
    // Parse command line arguments and the config file
    let cmd_args = parse_args()?;
    let config: config::Config;

    match config::parse_config(&cmd_args) {
        Ok(v) => config = v,
        _ => {
            println!("WARN : Failed to parse config file, defaulting to nma");
            config = config::backup_config()
        }
    };

    if config.as_cmd.is_empty() {
        // If no provided as config, default to NMA
        assemble(&cmd_args)?;
    } else {
        // Otherwise, use provided assembler command
        println!("Config Name:   {}", config.config_name);
        println!("Assembler CMD: {}", config.as_cmd);
    }

    Ok(())
}
