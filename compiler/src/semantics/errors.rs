use std::fmt::{Display, Formatter};
use super::{SemanticType};

pub enum SemanticError {
    UndefinedDatasource {
        name: String,
    },
    DuplicateDatasourceDeclaration {
        name: String,
    },
    UndefinedTable {
        name: String,
    },
    UndefinedColumn {
        table_name: String,
        column_name: String,
    },
    DatasourceReadonly {
        datasource_name: String,
        table_name: String,
    },
    IncompatibleColumnValue {
        table_name: String,
        column_name: String,
        expected: SemanticType,
        found: SemanticType,
    },
    NonPrimitiveColumnType {
        table_name: String,
        column_name: String,
    },
    DuplicateTableDefinition {
        name: String,
    },
    UndefinedFunction {
        name: String,
    },
    UndefinedMethod {
        receiver_type: SemanticType,
        method_name: String,
    },
    DuplicateFunctionDefinition {
        name: String,
    },
    UndefinedStruct {
        name: String,
    },
    DuplicateFieldInitialization {
        name: String,
    },
    IncompatibleStructInitialization {
        struct_name: String,
    },
    UndefinedStructFieldAccess {
        struct_type: SemanticType,
        field_name: String,
    },
    AnonymousStructFieldAccess {
        struct_type: SemanticType,
        field_name: String,
    },
    NonStructFieldAccess {
        sem_type: SemanticType,
        field_name: String,
    },
    HeterogeneousArray {
        type_a: SemanticType,
        type_b: SemanticType,
    },
    NonIntegralArrayIndex {
        index_type: SemanticType,
    },
    NonArrayIndex {
        sem_type: SemanticType,
    },
    UndefinedVariable {
        name: String,
    },
    DuplicateVariableDefinition {
        name: String,
    },
    AmbiguousVariableType {
        var_name: String,
        var_type: SemanticType
    },
    VoidVariableType {
        var_name: String,
    },
    VoidParameterType {
        function_name: String,
        param_name: String,
    },
    IncompatibleAssignment {
        var_name: String,
        var_type: SemanticType,
        expr_type: SemanticType
    },
    IncompatibleOperands {
        operation: String,
        left_type: SemanticType,
        right_type: SemanticType
    },
    MismatchingCallArity {
        function_name: String,
        expected: usize,
        found: usize
    },
    IncompatibleArgumentType {
        function_name: String,
        position: usize,
        expected: SemanticType,
        found: SemanticType
    },
    NotCallableType {
        found_type: SemanticType,
    },
    ReadonlyTableMutation {
        table_name: String,
        operation: &'static str,
    },
    // SELECT
    SelectDuplicateAlias {
        alias: String,
    },
    SelectIncompatibleCapture {
        capturing_struct: String,
    },
    InvalidLeftTable {
        left_table_name: String,
    },
    MismatchingJoinColumnTypes {
        left_table_name: String,
        left_column_name: String,
        left_column_type: SemanticType,
        right_table_name: String,
        right_column_name: String,
        right_column_type: SemanticType,
    },
    ExcludedTableInSelect {
        table_name: String,
    },
    // INSERT
    IncompatibleInsertData {
        table_name: String,
        found_type: SemanticType,
    },
    NonBoolCondition {
        found_type: SemanticType,
    },
    MistypedReturnValue {
        expected: SemanticType,
        found: SemanticType,
    },
    AmbiguousReturnType {
        return_type: SemanticType,
    },
    InexhaustiveReturnPaths {
        function_name: String,
    },
    InvalidMainSignature,
    MissingMainFunction,
    InvalidLoopLabel {
        label: String,
    },
    BreakOutsideLoop,
    ContinueOutsideLoop,
    NonIterableExpression {
        found_type: SemanticType,
    }
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SemanticError::UndefinedDatasource { name } => {
                write!(f, "Datasource {} is undefined", name)
            }
            SemanticError::DuplicateDatasourceDeclaration { name } => {
                write!(f, "Cannot redeclare existing datasource {}", name)
            }
            SemanticError::UndefinedTable { name } => {
                write!(f, "Table {} is undefined", name)
            }
            SemanticError::NonPrimitiveColumnType { table_name, column_name } => {
                write!(f, "Column {} of table {} must be of a primitive type", column_name, table_name)
            }
            SemanticError::UndefinedColumn { table_name, column_name } => {
                write!(f, "Table {} has no column named {}", table_name, column_name)
            }
            SemanticError::DatasourceReadonly { datasource_name, table_name } => {
                write!(f,
                    "Table {} must be declared read-only, as it is from a read-only datasource {}",
                    table_name, datasource_name)
            }
            SemanticError::IncompatibleColumnValue { table_name, column_name, expected, found } => {
                write!(f, "Value of type {} is incompatible with {} column {} of table {}", found, expected, column_name, table_name)
            }
            SemanticError::DuplicateTableDefinition { name } => {
                write!(f, "Cannot redefine existing table {}", name)
            }
            SemanticError::UndefinedFunction { name } => {
                write!(f, "Function {} is undefined", name)
            }
            SemanticError::UndefinedMethod { receiver_type, method_name } => {
                write!(f, "Type {} has no method named {}", receiver_type, method_name)
            }
            SemanticError::DuplicateFunctionDefinition { name } => {
                write!(f, "Cannot redefine existing function {}", name)
            }
            SemanticError::UndefinedStruct { name } => {
                write!(f, "Struct {} is undefined", name)
            }
            SemanticError::DuplicateFieldInitialization { name } => {
                write!(f, "Struct field {} is initialized multiple times", name)
            }
            SemanticError::IncompatibleStructInitialization { struct_name } => {
                write!(f, "Incompatible initialization of struct {}\n", struct_name)
            }
            SemanticError::UndefinedStructFieldAccess { struct_type, field_name } => {
                write!(f, "Struct type {} has no field named {}", struct_type, field_name)
            }
            SemanticError::AnonymousStructFieldAccess { struct_type, field_name } => {
                write!(f, "Cannot access field {} on anonymous struct type {}", field_name, struct_type)
            }
            SemanticError::NonStructFieldAccess { sem_type, field_name } => {
                write!(f, "Cannot access field {} on non-struct type {}", field_name, sem_type)
            }
            SemanticError::HeterogeneousArray { type_a, type_b } => {
                write!(f, "Array elements are not the same type; found {} and {}", type_a, type_b)
            }
            SemanticError::NonIntegralArrayIndex { index_type } => {
                write!(f, "Cannot index array with non-integral type {}", index_type)
            }
            SemanticError::NonArrayIndex { sem_type } => {
                write!(f, "Cannot index non-array type {}", sem_type)
            }
            SemanticError::UndefinedVariable { name } => {
                write!(f, "Variable {} is undefined", name)
            }
            SemanticError::DuplicateVariableDefinition { name } => {
                write!(f, "Variable {} is already defined in the current scope", name)
            }
            SemanticError::AmbiguousVariableType { var_name, var_type } => {
                write!(f, "Variable {} has an ambiguous type: {}", var_name, var_type)
            }
            SemanticError::VoidVariableType { var_name } => {
                write!(f, "Variable {} cannot have type void", var_name)
            },
            SemanticError::VoidParameterType { function_name, param_name } => {
                write!(f, "Parameter {} of function {} cannot have type void", param_name, function_name)
            },
            SemanticError::IncompatibleAssignment { var_name, var_type, expr_type } => {
                write!(f, "Cannot assign value of type {} to variable {} of type {}", expr_type, var_name, var_type)
            }
            SemanticError::IncompatibleOperands { operation, left_type, right_type } => {
                write!(f, "Operands of types {} and {} are incompatible under {}", left_type, right_type, operation)
            }
            SemanticError::MismatchingCallArity { function_name, expected, found } => {
                write!(f, "Function {} expects {} arguments but {} were provided", function_name, expected, found)
            }
            SemanticError::IncompatibleArgumentType { function_name, position, expected, found } => {
                write!(f, "Cannot pass {} argument to {} argument {} of function {}", found, expected, position, function_name)
            }
            SemanticError::NotCallableType { found_type } => {
                write!(f, "Type {} cannot be called as a function", found_type)
            }
            SemanticError::ReadonlyTableMutation { table_name, operation } => {
                write!(f, "Cannot perform {} on read-only table {}", operation, table_name)
            }
            SemanticError::SelectDuplicateAlias { alias } => {
                write!(f, "Duplicate alias {} in SELECT query", alias)
            }
            SemanticError::SelectIncompatibleCapture { capturing_struct } => {
                write!(f, "Capturing struct {} is incompatible with columns in SELECT query", capturing_struct)
            }
            SemanticError::InvalidLeftTable { left_table_name } => {
                write!(f, "Invalid left table {} in JOIN clause", left_table_name)
            }
            SemanticError::MismatchingJoinColumnTypes { left_table_name, left_column_name, left_column_type, right_table_name, right_column_name, right_column_type } => {
                write!(f, "Cannot join {}.{} of type {} with {}.{} of type {}",
                    left_table_name, left_column_name, left_column_type,
                    right_table_name, right_column_name, right_column_type)
            }
            SemanticError::ExcludedTableInSelect { table_name } => {
                write!(f, "Table {} cannot be SELECTed from as is not a part of the query", table_name)
            }
            SemanticError::IncompatibleInsertData { table_name, found_type } => {
                write!(f, "Expected {} row in INSERT, got {} instead", table_name, found_type)
            }
            SemanticError::NonBoolCondition { found_type } => {
                write!(f, "Condition expression must be of boolean type, found {}", found_type)
            }
            SemanticError::MistypedReturnValue { expected, found } => {
                write!(f, "Return value of type {} does not match expected type {}", found, expected)    
            }
            SemanticError::AmbiguousReturnType { return_type } => {
                write!(f, "Closure has ambiguous return type: {}", return_type)
            }
            SemanticError::InexhaustiveReturnPaths { function_name } => {
                write!(f, "Not all code paths in function {} return a value", function_name)
            }
            SemanticError::InvalidMainSignature => {
                write!(f, "Function main must return an integer and accept no parameters")
            }
            SemanticError::MissingMainFunction => {
                write!(f, "Program must contain a main function")
            }
            SemanticError::InvalidLoopLabel { label } => {
                write!(f, "No loop with label {} exists", label)
            }
            SemanticError::BreakOutsideLoop => {
                write!(f, "break statement used outside of a loop")
            }
            SemanticError::ContinueOutsideLoop => {
                write!(f, "continue statement used outside of a loop")
            }
            SemanticError::NonIterableExpression { found_type } => {
                write!(f, "Expression of type {} cannot be iterated over", found_type)
            }
        }
    }
}