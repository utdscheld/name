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
standard_args = { 
   instruction_arg ~ ("," ~ WHITESPACE* ~ instruction_arg){, 2}
}
mem_access_args = { instruction_arg ~ "," ~ instruction_arg ~ "(" ~ instruction_arg ~ ")" }
instruction_args = { mem_access_args | standard_args }
instruction = { ident ~ instruction_args }

vernacular = {instruction | label}
vernaculars = { vernacular* }
"#]
pub struct MipsParser;
