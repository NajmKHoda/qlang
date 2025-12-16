use std::io::Read;
use std::io::Error as IOError;
use std::fs::File;
use lalrpop_util::lalrpop_mod;

mod tokens;
mod codegen;

lalrpop_mod!(pub simple);

fn main() -> Result<(), IOError> {
    let parser = simple::ProgramParser::new();
    let mut source = String::new();
    let mut file = File::open("main.ql")?;
    file.read_to_string(&mut source)?;

    let program = parser.parse(&source).map_err(|e| {
        eprintln!("Failed to parse main.ql: {e}");
        IOError::new(std::io::ErrorKind::InvalidData, "Parsing failed")
    })?;

    codegen::gen_code(&program).map_err(|e| {
        eprintln!("Failed to build main.ql: {e}");
        IOError::new(std::io::ErrorKind::InvalidData, "Building failed")
    })?;

    Ok(())
}
