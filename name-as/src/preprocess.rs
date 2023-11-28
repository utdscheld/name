pub fn preprocess(mut input: String, input_as: &str) -> Result<String, String> {
    let mut out = Vec::new();

    let mut eqv: HashMap<String, String> = HashMap::new();
    let mut mac: HashMap<String, (Vec<String>, String)> = HashMap::new();

    let mut collecting_macro = false;
    let mut macro_name: String = "".to_string();
    let mut macro_args: Vec<String> = Vec::new();
    let mut macro_buf = String::new();

    let input_path = PathBuf::from(input_as);
    let base_folder = input_path.parent().unwrap();

    let regex = Regex::new(".include \"(?P<include_fn>.*)\"").unwrap();
    for (_, [include_fn]) in regex.captures_iter(&input.clone()).map(|c| c.extract()) {
        let file_contents: String = match fs::read_to_string(base_folder.join(include_fn)) {
            Ok(v) => v,
            Err(_) => return Err("Failed to read input file contents".to_string()),
        };
        input = regex.replace_all(&input, file_contents).to_string();
    }

    for mut line in input.lines() {
        line = line.trim();

        // Filter comments
        if let Some(pos) = line.find('#') {
            line = &line[0..pos]; // Remove the comment part
        }

        if line.is_empty() {
            continue;
        }

        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        // Handle .eqv directives
        if tokens[0] == ".eqv" {
            eqv.insert(tokens[1].clone(), tokens[2].clone());
            continue;
        }

        // Handle .macro directives
        if tokens[0] == ".macro" {
            collecting_macro = true;
            macro_name = tokens[1].clone();
            // Remove the '(', ')', and ',',
            macro_args = scan_macro_args(&tokens[2..]);
            continue;
        } else if tokens[0] == ".end_macro" {
            collecting_macro = false;
            mac.insert(macro_name.clone(), (macro_args.clone(), macro_buf.clone()));
            continue;
        }

        // Replace via eqv
        let mut tokens: Vec<String> = tokens
            .iter()
            .map(|token| {
                let mut subbed_token = token.to_string();
                for eqv_key in eqv.keys() {
                    subbed_token = subbed_token.replace(eqv_key, eqv.get(eqv_key).unwrap())
                }
                subbed_token.to_owned()
            })
            .collect::<Vec<String>>();

        // Replace via macro
        if let Some(scanned_macro) = mac.get(tokens[0].as_str()) {
            let input_args: Vec<String> = scan_macro_args(&tokens[1..]);
            tokens.clear();
            for macro_line in scanned_macro.1.lines() {
                let mut subbed_in_line = macro_line.to_string().clone();
                for (index, arg) in scanned_macro.0.iter().enumerate() {
                    subbed_in_line = subbed_in_line.replace(arg, &input_args[index]);
                }
                out.push(subbed_in_line);
            }
            continue;
        }

        if collecting_macro {
            macro_buf += (tokens.join(" ") + "\n").as_str();
        } else {
            out.push(tokens.join(" "));
        }
    }

    Ok(out.join("\n"))
}
