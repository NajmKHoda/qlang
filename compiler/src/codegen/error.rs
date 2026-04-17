use std::fmt;
use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;

pub enum CodeGenError {
    BuilderError(BuilderError),
    ModuleVerificationError(LLVMString),
    TargetError(LLVMString),
    TargetMachineError,
    TargetMachineWriteError(LLVMString),
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::BuilderError(err) => write!(f, "Builder error:\n\t{err}"),
            CodeGenError::ModuleVerificationError(err) => write!(f, "Module verification error:\n\t{err}"),
            CodeGenError::TargetError(err) => write!(f, "Target error:\n\t{err}"),
            CodeGenError::TargetMachineError => write!(f, "Target machine creation error"),
            CodeGenError::TargetMachineWriteError(err) => write!(f, "Target machine write to file error:\n\t{err}"),
        }
    }
}

impl From<BuilderError> for CodeGenError {
    fn from(err: BuilderError) -> Self { CodeGenError::BuilderError(err) }
}
