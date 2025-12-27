use std::fmt;
use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;

pub enum CodeGenError {
    UnexpectedTypeError,
    UndefinedVariableError(String),
    UndefinedLoopLabelError(String),
    UndefinedTableError(String),
    UndefinedTableColumnError(String, String),
    UndefinedMethodError(String, String),
    DuplicateColumnAssignmentError(String, String),
    MissingColumnAssignmentError(String),

    BadLoopControlError,
    DuplicateDefinitionError(String),
    BadFunctionCallError(String),
    BadArgumentMutationError(String, String),
    InexhaustiveReturnError(String),
    MissingMainError,
    BadMainSignatureError,

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
            CodeGenError::UndefinedLoopLabelError(name) => write!(f, "Undefined loop label: {name}"),
            CodeGenError::UndefinedTableError(name) => write!(f, "Undefined table: {name}"),
            CodeGenError::UndefinedTableColumnError(column_name, table_name) => write!(f, "Undefined column {column_name} in table {table_name}"),
            CodeGenError::UndefinedMethodError(method_name, type_name) => write!(f, "Type {type_name} has no method \"{method_name}\""),
            CodeGenError::DuplicateColumnAssignmentError(column_name, table_name) => write!(f, "Duplicate assignment to column {column_name} in table {table_name}"),
            CodeGenError::MissingColumnAssignmentError(table_name) => write!(f, "Missing assignment to one or more columns in table {table_name}"),

            CodeGenError::BadLoopControlError => write!(f, "No loop to break/continue from"),
            CodeGenError::DuplicateDefinitionError(name) => write!(f, "Duplicate definition for {name}"),
            CodeGenError::BadFunctionCallError(name) => write!(f, "Bad function call: {name}"),
            CodeGenError::BadArgumentMutationError(fn_name, arg) => write!(f, "Attempt to mutate argument {arg} of {fn_name}"),
            CodeGenError::InexhaustiveReturnError(fn_name) => write!(f, "Not all paths return a value in function {fn_name}"),
            CodeGenError::MissingMainError => write!(f, "Missing main function"),
            CodeGenError::BadMainSignatureError => write!(f, "Main function has invalid signature"),

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
