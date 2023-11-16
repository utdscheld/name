extern crate pest;
extern crate pest_derive;
//use name_const::LineInfo;

pub mod args;
pub mod config;

pub mod nma;
pub mod parser;

use args::parse_args;
use nma::assemble;
use std::process::Command;

fn main() -> Result<(), String> {
    // Parse command line arguments and the config file
    let cmd_args = parse_args()?;

    let config: config::Config = match config::parse_config(&cmd_args) {
        Ok(v) => v,
        _ => {
            println!("WARN : Failed to parse config file, defaulting to nma");
            config::backup_config()
        }
    };

    if config.as_cmd.is_empty() {
        // If no provided as config, default to NMA
        assemble(&cmd_args)?;
    } else {
        // Otherwise, use provided assembler command
        println!("Config Name:   {}", config.config_name);
        println!("Assembler CMD: {:?}", config.as_cmd);

        for full_cmd in &config.as_cmd {
            let split_cmd: Vec<&str> = full_cmd.split_whitespace().collect();

            match Command::new(split_cmd[0]).args(&split_cmd[1..]).output() {
                Ok(output) => {
                    if output.status.success() {
                        if !&output.stdout.is_empty() {
                            println!(
                                "CMD {}\n{}",
                                full_cmd,
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                    } else if !&output.stderr.is_empty() {
                        eprintln!(
                            "CMD {}\n{}",
                            full_cmd,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(err) => {
                    eprintln!("CMD {}\nError: {}", full_cmd, err);
                    return Err("Failed to run assembler command".to_string());
                }
            }
        }
    }

    Ok(())
}
