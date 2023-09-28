use pest_derive::Parser;

#[derive(Parser)]
#[grammar_inline = r#"
alpha = _{ 'a'..'z' | 'A'..'Z' }
digit = _{ '0'..'9' }
WHITESPACE = _{ " " | NEWLINE }

ident = @{ alpha ~ (alpha | digit)* }

label = { ident ~ ":" }

register = @{ "$" ~ ident }
instruction_arg = @{ ident | register | digit+ }
standard_args = _{ 
   instruction_arg ~ ("," ~ WHITESPACE* ~ instruction_arg){, 2}
}
mem_access_args = _{ instruction_arg ~ "," ~ instruction_arg ~ "(" ~ instruction_arg ~ ")" }
instruction_args = _{ mem_access_args | standard_args }
instruction = { ident ~ instruction_args }

vernacular = { (instruction | label)* }
"#]
pub struct MipsParser;

#[derive(Debug)]
pub enum MipsAST<'a> {
    Label(&'a str),
    Instruction(&'a str, Vec<&'a str>),
    Sequence(Vec<MipsAST<'a>>),
}

pub fn print_ast(ast: MipsAST) {
    match ast {
        MipsAST::Label(s) => println!("{}:", s),
        MipsAST::Instruction(mnemonic, args) => println!("\t{} {}", mnemonic, args.join(", ")),
        MipsAST::Sequence(v) => {
            for sub_ast in v {
                print_ast(sub_ast)
            }
        }
    }
}
