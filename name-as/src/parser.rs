use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar_inline = r##"
alpha = _{ 'a'..'z' | 'A'..'Z' }
digit = _{ '0'..'9' }
hex   = _{ "0x" ~ (digit | 'a'..'f' | 'A'..'F')+ }
binary = _{ "0b" ~ ('0'..'1')+ }
decimal = _{ digit+ }
float = _{ digit+ ~ "." ~ digit* }
integer = { hex | binary | decimal }
number = _{ integer | float }
string = { "\"" ~ ANY* ~ "\""}
WHITESPACE = _{ " " | NEWLINE }

ident = @{ alpha ~ (alpha | digit)* }

label = { ident ~ ":" }

macro_param = @{ "%" ~ ident }
register = @{ "$" ~ ident }
instruction_arg = @{ ident | macro_param | register | integer }
standard_args = _{ 
   instruction_arg ~ ("," ~ WHITESPACE* ~ instruction_arg){, 2}
}
mem_access_args = _{ instruction_arg ~ "," ~ integer? ~ "(" ~ instruction_arg ~ ")" }
instruction_args = _{ mem_access_args | standard_args }
instruction = { ident ~ instruction_args }

no_arg_directives = @{
    "text"
    | "data"
    | "end_macro"
}

num_arg_directives = @{ 
    "align"
    | "byte"
    | "double"
    | "float"
    | "half"
    | "kdata"
    | "ktext"
    | "space"
    | "word"
}

str_arg_directives = @{
    "ascii"
    | "asciiz"
    | "extern"
    | "eqv"
    | "globl"
    | "include"
    | "set"
}

directive = {
    "." ~ ((
        str_arg_directives ~ (
            ident ~ number
            | ident
            | ident ~ string
            | string
        )
    ) | (
        num_arg_directives ~ 
        (
            number ~ ("," ~ WHITESPACE* ~ number)*
        )*
    ) | (
        no_arg_directives
    ))
}

vernacular = { SOI ~ ((directive | instruction | label ))* }
"##]
pub struct MipsParser;

#[derive(Debug, Clone)]
pub enum MipsCST<'a> {
    Label(&'a str),
    Instruction(&'a str, Vec<&'a str>),
    Directive(&'a str, Vec<&'a str>),
    Sequence(Vec<MipsCST<'a>>),
}

pub fn parse_rule(pair: Pair<Rule>) -> MipsCST {
    match pair.as_rule() {
        Rule::label => MipsCST::Label(pair.into_inner().next().unwrap().as_str()),
        Rule::instruction => {
            let mut inner = pair.into_inner();
            let opcode = inner.next().unwrap().as_str();
            let args = inner.clone().map(|p| p.as_str()).collect::<Vec<&str>>();
            MipsCST::Instruction(opcode, args)
        }
        Rule::directive => {
            let mut inner = pair.into_inner();
            let directive = inner.next().unwrap().as_str();
            let args = inner.clone().map(|p| p.as_str()).collect::<Vec<&str>>();
            MipsCST::Directive(directive, args)
        }
        Rule::vernacular => MipsCST::Sequence(pair.into_inner().map(parse_rule).collect()),
        _ => {
            println!("Unreachable: {:?}", pair.as_rule());
            unreachable!()
        }
    }
}

pub fn cst_map(cst: &MipsCST, f: fn(&MipsCST) -> ()) {
    match cst {
        MipsCST::Sequence(v) => {
            let _ = v.iter().map(f);
        }
        _ => f(cst),
    }
}

pub fn print_cst(cst: &MipsCST) {
    match cst {
        MipsCST::Label(s) => println!("{}:", s),
        MipsCST::Instruction(mnemonic, args) => println!("\t{} {}", mnemonic, args.join(", ")),
        MipsCST::Directive(directive, args) => println!(".{} {}", directive, args.join(", ")),
        MipsCST::Sequence(v) => {
            for sub_cst in v {
                print_cst(sub_cst)
            }
        }
    }
}

pub fn instr_to_str(mnemonic: &str, args: &[&str]) -> String {
    format!("{} {}", mnemonic, args.join(" "))
}
