use std::io::Read;
use std::env;
use std::fs::File;
use std::env::args;
use std::process::{Command, ExitCode};
use lalrpop_util::lalrpop_mod;

use crate::semantics::SemanticGen;
use crate::codegen::CodeGen;

mod tokens;
mod semantics;
mod codegen;

lalrpop_mod!(pub grammar);

fn main() -> ExitCode {
    let args: Vec<String> = args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <source-file> <object-file>", args[0]);
        return ExitCode::FAILURE;
    }

    let source_filepath = &args[1];
    let obj_filepath = &args[2];

    let caller_cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
            eprintln!("Failed to read current working directory:\n\t{e}");
            return ExitCode::FAILURE;
        }
    };
    let obj_path = caller_cwd.join(obj_filepath);

    let compiler_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = match compiler_root.parent() {
        Some(path) => path.join("out"),
        None => {
            eprintln!("Failed to resolve project root from CARGO_MANIFEST_DIR");
            return ExitCode::FAILURE;
        }
    };

    let parser = grammar::ProgramParser::new();
    let mut source = String::new();

    let mut file = match File::open(source_filepath) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {source_filepath}:\n\t{e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = file.read_to_string(&mut source) {
        eprintln!("Failed to read {source_filepath}:\n\t{e}");
        return ExitCode::FAILURE;
    }

    // Parsing
    let program = match parser.parse(&source) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parsing failed on {source_filepath}:\n\t{e}");
            return ExitCode::FAILURE;
        }
    };

    // Semantic analysis
    let semantic_program = match SemanticGen::gen_semantic(&program) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Analysis failed on {source_filepath}:\n\t{e}");
            return ExitCode::FAILURE;
        }
    };

    // Code generation
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("Failed to create out directory:\n\t{e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = CodeGen::gen_code(&semantic_program, &out_dir) {
        eprintln!("Failed to build {source_filepath}:\n\t{e}");
        return ExitCode::FAILURE;
    }

    let result = Command::new("cc")
        .arg(out_dir.join("main.o"))
        .arg(out_dir.join("runtime.o"))
        .arg("-o")
        .arg(&obj_path)
        .arg("-lsqlite3")
        .status();
    match result {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("Failed to link {}:\n\tlinker exited with {status}", obj_path.display());
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("Failed to link {}:\n\t{e}", obj_path.display());
            return ExitCode::FAILURE;
        }
    }
    
    ExitCode::SUCCESS
}
