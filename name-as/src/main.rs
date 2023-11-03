extern crate pest;
extern crate pest_derive;
extern crate tempfile;

pub mod args;
pub mod config;
pub mod lineinfo;

pub mod nma;
pub mod parser;

use args::parse_args;
use nma::assemble;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

fn main() -> Result<(), String> {
    // Parse command line arguments and the config file
    let cmd_args = parse_args()?;

    let mut config: config::Config = match config::parse_config(&cmd_args) {
        Ok(v) => v,
        _ => {
            println!("WARN : Failed to parse config file, defaulting to nma");
            config::backup_config()
        }
    };

    if cmd_args.only_preprocess {
        if let Ok(contents) = fs::read_to_string(cmd_args.input_as) {
            println!("{}", nma::preprocess(contents));
        }
        return Ok(());
    }

    if config.as_cmd.is_empty() {
        // If no provided as config, default to NMA
        assemble(&config, &cmd_args)?;
    } else {
        let mut temp_file: NamedTempFile = match NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };

        if config.preprocess {
            let input_file = match fs::read_to_string(cmd_args.input_as) {
                Ok(v) => v,
                Err(_) => return Err("Failed to read input file contents".to_string()),
            };

            if let Err(e) = temp_file.write_all(nma::preprocess(input_file).as_bytes()) {
                return Err(e.to_string());
            }

            let temp_fn = match temp_file.path().to_str() {
                Some(f) => f,
                None => return Err("Failed to get tempfile name".to_string()),
            };

            // Replace via eqv
            config.as_cmd = config
                .as_cmd
                .iter()
                .map(|cmd| cmd.replace("{PREPROCESSED_AS}", temp_fn))
                .collect();
        }

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
                        return Err("Assembler failed".to_string());
                    }
                }
                Err(err) => {
                    eprintln!("CMD {}\nError: {}", full_cmd, err);
                    return Err("Failed to run assembler command".to_string());
                }
            }
        }

        if let Err(e) = temp_file.close() {
            return Err(e.to_string());
        }
    }

    Ok(())
}
