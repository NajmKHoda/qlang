use std::io::Read;
use std::io::{Error as IOError, ErrorKind};
use std::fs::File;
use std::env::args;
use std::process::Command;
use lalrpop_util::lalrpop_mod;

mod tokens;
mod codegen;

lalrpop_mod!(pub grammar);

fn main() -> Result<(), IOError> {
    let args: Vec<String> = args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <source-file> <object-file>", args[0]);
        return Err(IOError::new(ErrorKind::InvalidInput, "Not enough arguments"));
    }

    let source_filepath = &args[1];
    let obj_filepath = &args[2];

    let parser = grammar::ProgramParser::new();
    let mut source = String::new();
    let mut file = File::open(source_filepath)?;
    file.read_to_string(&mut source)?;

    let program = parser.parse(&source).map_err(|e| {
        eprintln!("Failed to parse {source_filepath}: {e}");
        IOError::new(ErrorKind::InvalidData, "Parsing failed")
    })?;

    codegen::gen_code(&program).map_err(|e| {
        eprintln!("Failed to build {source_filepath}: {e}");
        IOError::new(ErrorKind::InvalidData, "Building failed")
    })?;

    Command::new("cc")
        .args(&["out/main.o", "out/runtime.o", "-o", obj_filepath, "-lsqlite3"])
        .status()?;

    Ok(())
}
