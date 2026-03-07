mod lexer;
mod enums;

use std::{env, fs};
use crate::lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];
    let file_content = fs::read_to_string(file_path);

    match file_content {
        Ok(content) => {
            let mut runtime = Lexer::new(content);
            runtime.start()

        }
        Err(error) => { panic!("{}", error); }
    }
}

fn error(line: u8, message: &str) {
    panic!("Error on line {}: {}", line, message);
}


