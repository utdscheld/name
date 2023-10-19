use pest::iterators::Pair;
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

#[derive(Debug, Clone)]
pub enum MipsCST<'a> {
    Label(&'a str),
    Instruction(&'a str, Vec<&'a str>),
    Sequence(Vec<MipsCST<'a>>),
}

pub fn parse_rule(pair: Pair<Rule>) -> MipsCST {
    match pair.as_rule() {
        Rule::vernacular => MipsCST::Sequence(pair.into_inner().map(parse_rule).collect()),
        Rule::label => MipsCST::Label(pair.into_inner().next().unwrap().as_str()),
        Rule::instruction => {
            let mut inner = pair.into_inner();
            let opcode = inner.next().unwrap().as_str();
            let args = inner.clone().map(|p| p.as_str()).collect::<Vec<&str>>();
            MipsCST::Instruction(opcode, args)
        }
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
