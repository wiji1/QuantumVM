use quantum_vm::{run_program, ExecutionResult, RunConfig, SourceCache, ErrorReporter};
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (file_path, _json_input_path, _cli_inputs) = parse_cli_args(&args);

    let source = std::fs::read_to_string(&file_path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file_path, e);
        std::process::exit(1);
    });

    let cache = SourceCache::new();
    cache.add_source(file_path.clone(), source.clone());

    let reporter = ErrorReporter::new(cache);

    let script_path = PathBuf::from(&file_path);
    let working_dir = script_path.parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    // TODO: Support JSON inputs and CLI inputs through RunConfig
    let result = run_program(&source, RunConfig {
        working_dir: Some(working_dir),
        ..Default::default()
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
        ExecutionResult::ParseError(errors) => {
            eprintln!("Parsing failed with {} error(s):\n", errors.len());
            reporter.report_parse_errors(&file_path, &errors);
            std::process::exit(1);
        }
        ExecutionResult::TypeCheckError(errors) => {
            eprintln!("Type checking failed with {} error(s):\n", errors.len());
            reporter.report_type_errors(&file_path, &errors);
            std::process::exit(1);
        }
        ExecutionResult::RuntimeError(e) => {
            reporter.report_runtime_error(&file_path, &e);
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



