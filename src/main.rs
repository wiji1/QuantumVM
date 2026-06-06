mod lexer;
mod parser;
mod interpreter;
mod type_checker;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_checker::{TypeCheckConfig, TypeChecker};
use std::{env, fs};
use std::path::PathBuf;
use crate::type_checker::diagnostics::DiagnosticSeverity;

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
            let program = parser.start(true).expect("Parse Error");

            let mut type_checker = TypeChecker::new(TypeCheckConfig::default());
            let result = type_checker.check_program(&program);

            if !result.success {
                eprintln!("Type checking failed with {} error(s):",
                    result.diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Error)).count());
                for diag in result.diagnostics { eprintln!("{}", diag); }
                std::process::exit(1);
            }

            let mut interpreter = Interpreter::new(program, script_dir);
            //TODO: Set input values
            interpreter.start();

            //TODO: display output values
            println!("Outputs: {:?}", interpreter.get_outputs().iter());
            println!("StateVector: {:?}", interpreter.get_state_vector())

        }
        Err(error) => { panic!("{}", error); }
    }
}


