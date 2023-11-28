use std::env;

#[derive(Debug, Clone)]
pub struct Args {
    pub config_fn: String,
    pub input_as: String,
    pub output_as: String,
    pub line_info: bool,
    pub only_preprocess: bool,
}

fn help() {
    println!("Usage: name [OPTIONS] CONFIG INPUT OUTPUT\n");
    println!("Required:");
    println!("  CONFIG       A toml configuration file, examples");
    println!("               are provided in configs/");
    println!("  INPUT_AS     An input assembly file");
    println!("  OUTPUT_AS    An output assembled file");
    println!("Optional:");
    println!("  --lineinfo");
    println!("   -l          Enables line information export");
    println!("  --preprocess Only preprocess the input assembly file");
}

pub fn parse_args() -> Result<Args, &'static str> {
    let mut args: Args = Args {
        config_fn: String::new(),
        input_as: String::new(),
        output_as: String::new(),
        line_info: false,
        only_preprocess: false,
    };
    let args_strings: Vec<String> = env::args().collect();

    if args_strings.len() < 4 {
        help();
        return Err("Incorrect number of arguments");
    }

    let mut arg_index = 1;
    for arg in args_strings.iter().skip(1) {
        let mut parsed_option = true;
        match arg.as_str() {
            "-l" | "--lineinfo" => args.line_info = true,
            "--preprocess" => args.only_preprocess = true,
            _ => parsed_option = false,
        };
        if parsed_option {
            continue;
        }

        match arg_index {
            1 => args.config_fn = arg.to_string(),
            2 => args.input_as = arg.to_string(),
            3 => args.output_as = arg.to_string(),
            _ => return Err("Argument out of bounds"),
        }

        arg_index += 1;
    }

    if args.config_fn == String::new() {
        return Err("Expected a configuration file but found none");
    } else if args.input_as == String::new() {
        return Err("Expected an input assembly file but found none");
    } else if args.output_as == String::new() {
        return Err("Expected an output assembly file but found none");
    }

    Ok(args)
}
