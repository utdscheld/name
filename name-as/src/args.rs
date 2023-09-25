use std::env;

#[derive(Debug)]
pub struct Args {
    pub config_fn: String,
    pub input_as: String,
    pub output_as: String,
}

fn help() {
    println!("Usage: name CONFIG INPUT OUTPUT\n");
    println!("Required:");
    println!("  CONFIG       A toml configuration file, examples");
    println!("               are provided in configs/");
    println!("  INPUT_AS     An input assembly file");
    println!("  OUTPUT_AS    An output assembled file");
}

pub fn parse_args() -> Result<Args, &'static str> {
    let mut args: Args = Args {
        config_fn: String::new(),
        input_as: String::new(),
        output_as: String::new(),
    };
    let args_strings: Vec<String> = env::args().collect();

    if args_strings.len() != 4 {
        help();
        return Err("Incorrect number of arguments");
    }

    for (i, arg) in args_strings.iter().enumerate().skip(1) {
        match i {
            1 => args.config_fn = arg.to_string(),
            2 => args.input_as = arg.to_string(),
            3 => args.output_as = arg.to_string(),
            _ => return Err("Argument out of bounds"),
        }
    }

    Ok(args)
}
