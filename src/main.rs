mod lexer;
mod parser;
mod interpreter;

use std::{env, fs};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];
    let file_content = fs::read_to_string(file_path);

    match file_content {
        Ok(content) => {
            let mut lexer = Lexer::new(content);
            lexer.start();
            
            let lexer_output = lexer.tokens;
            let mut parser = Parser::new(lexer_output);
            let program = parser.start().expect("TODO: panic message");
            
            let mut interpreter = Interpreter::new(program);
            interpreter.start();
        }
        Err(error) => { panic!("{}", error); }
    }
}

fn error(line: u8, message: &str) {
    panic!("Error on line {}: {}", line, message);
}


