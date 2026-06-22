use quantum_vm::{run_file, ExecutionResult};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (file_path, _json_input_path, _cli_inputs) = parse_cli_args(&args);

    // TODO: Support JSON inputs and CLI inputs through RunConfig
    let result = run_file(&file_path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file_path, e);
        std::process::exit(1);
    });

    match result {
        ExecutionResult::Success { outputs } => {
            if !outputs.is_empty() {
                println!("\nOutputs:");
                for (name, value) in outputs {
                    println!("  {} = {:?}", name, value);
                }
            }
            std::process::exit(0);
        }
        ExecutionResult::ParseError(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
        ExecutionResult::TypeCheckError(errors) => {
            eprintln!("Type checking failed with {} error(s):", errors.len());
            for error in errors {
                eprintln!("{}", error);
            }
            std::process::exit(1);
        }
        ExecutionResult::RuntimeError(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(101);
        }
    }
}

fn parse_cli_args(args: &[String]) -> (String, Option<String>, Vec<(String, String)>) {
    if args.len() < 2 {
        eprintln!("Usage: {} <file.qasm> [--inputs <file.json>] [--input key=value ...]", args[0]);
        std::process::exit(1);
    }

    let file_path = args[1].clone();
    let mut json_input_path = None;
    let mut cli_inputs = Vec::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--inputs" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --inputs requires a file path");
                    std::process::exit(1);
                }
                json_input_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--input" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --input requires a key=value argument");
                    std::process::exit(1);
                }
                let input_str = &args[i + 1];
                if let Some((key, value)) = input_str.split_once('=') {
                    cli_inputs.push((key.to_string(), value.to_string()));
                    i += 2;
                } else {
                    eprintln!("Error: --input requires format key=value, got: {}", input_str);
                    std::process::exit(1);
                }
            }
            unknown => {
                eprintln!("Error: Unknown argument: {}", unknown);
                std::process::exit(1);
            }
        }
    }

    (file_path, json_input_path, cli_inputs)
}



