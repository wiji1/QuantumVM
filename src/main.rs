mod lexer;
mod parser;
mod interpreter;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::{env, fs};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];
    let file_content = fs::read_to_string(file_path);

    let script_path = PathBuf::from(file_path);
    let script_dir = script_path.parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    match file_content {
        Ok(content) => {
            let mut lexer = Lexer::new(content);
            lexer.start();
            
            let lexer_output = lexer.tokens;
            let mut parser = Parser::new(lexer_output);
            let program = parser.start().expect("TODO: panic message");
            
            let mut interpreter = Interpreter::new(program, script_dir);
            //TODO: Set input values
            interpreter.start();

            //TODO: display output values
            println!("Outputs: {:?}", interpreter.get_outputs().values());

        }
        Err(error) => { panic!("{}", error); }
    }
}

fn error(line: u8, message: &str) {
    panic!("Error on line {}: {}", line, message);
}


