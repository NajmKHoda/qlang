use std::fmt;
use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;

pub enum CodeGenError {
    UnexpectedTypeError,
    UndefinedVariableError(String),
    DuplicateDefinitionError(String),

    BuilderError(BuilderError),
    ModuleVerificationError(LLVMString),
    TargetError(LLVMString),
    TargetMachineError,
    TargetMachineWriteError,
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::UnexpectedTypeError => write!(f, "Unexpected type encountered"),
            CodeGenError::UndefinedVariableError(name) => write!(f, "Undefined variable: {name}"),
            CodeGenError::DuplicateDefinitionError(name) => write!(f, "Duplicate definition for {name}"),
            CodeGenError::BuilderError(err) => write!(f, "Builder error: {err}"),
            CodeGenError::ModuleVerificationError(err) => write!(f, "Module verification error: {err}"),
            CodeGenError::TargetError(err) => write!(f, "Target error: {err}"),
            CodeGenError::TargetMachineError => write!(f, "Target machine creation error"),
            CodeGenError::TargetMachineWriteError => write!(f, "Target machine write to file error"),
        }
    }
}

impl From<BuilderError> for CodeGenError {
    fn from(err: BuilderError) -> Self { CodeGenError::BuilderError(err) }
}
