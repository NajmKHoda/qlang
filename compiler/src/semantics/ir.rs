#![allow(dead_code)]

use std::{collections::HashMap};

use crate::semantics::control_flow::SemanticBlock;

use super::*;

pub enum SemanticStatement {
    VariableDeclaration {
        variable_id: u32,
        init_expr: SemanticExpression,
    },
    VariableAssignment {
        variable_id: u32,
        expr: SemanticExpression,
    },
    LoneExpression(SemanticExpression),
    Conditional {
        branches: Vec<SemanticConditionalBranch>,
        else_branch: Option<SemanticBlock>,
    },
    ConditionalLoop {
        condition: SemanticExpression,
        body: SemanticBlock,
        id: u32,
    },
    Transaction {
        body: SemanticBlock,
        rollback_body: SemanticBlock,
        id: u32,
    },
    Return(Option<u32>),
    Release(u32),
    Break(u32),
    Continue(u32),
    DropVariable(u32)
}

pub struct SemanticConditionalBranch {
    pub condition: SemanticExpression,
    pub body: SemanticBlock,
}

pub struct SemanticExpression {
    pub kind: SemanticExpressionKind,
    pub sem_type: SemanticType,
    pub ownership: Ownership,
}

pub enum SemanticExpressionKind {
    IntegerLiteral(i32),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Struct(HashMap<String, SemanticExpression>),
    Array(Vec<SemanticExpression>),
    Closure {
        closure_id: u32,
        error_drops: Vec<u32>,
    },
    Variable(u32),
    StructField {
        struct_expr: Box<SemanticExpression>,
        index: u32,
    },
    ArrayIndex {
        array_expr: Box<SemanticExpression>,
        index_expr: Box<SemanticExpression>,
    },
    Range {
        start: Option<Box<SemanticExpression>>,
        end: Option<Box<SemanticExpression>>,
        inclusive: bool,
        step: Option<Box<SemanticExpression>>,
    },
    Add {
        left: Box<SemanticExpression>,
        right: Box<SemanticExpression>
    },
    Subtract {
        left: Box<SemanticExpression>,
        right: Box<SemanticExpression>
    },
    Compare {
        left: Box<SemanticExpression>,
        right: Box<SemanticExpression>,
        op: ComparisonType
    },
    DirectFunctionCall {
        function_id: u32,
        args: Vec<SemanticExpression>,
        error_drops: Vec<u32>,
    },
    IndirectFunctionCall {
        function_expr: Box<SemanticExpression>,
        args: Vec<SemanticExpression>,
        error_drops: Vec<u32>,
    },
    BuiltinFunctionCall {
        function: BuiltinFunction,
        args: Vec<SemanticExpression>,
    },
    BuiltinMethodCall {
        receiver: Box<SemanticExpression>,
        method: BuiltinMethod,
        args: Vec<SemanticExpression>,
    },
    ImmediateQuery {
        query: SemanticQuery,
        error_drops: Vec<u32>,
    },
}

#[derive(Clone, Copy)]
pub enum BuiltinFunction {
    PrintString,
    PrintInteger,
    PrintFloat,
    PrintBool,
    InputInteger,
    InputString,
    Zip,
    Concat,
}

#[derive(Clone, Copy)]
pub enum BuiltinMethod {
    ArrayLength,
    ArrayAppend,
    ArrayPop,
    ArrayIter,
    IteratorNext,
    IteratorHasNext,
    IteratorCollect,
}

pub enum SemanticQuery {
    Select {
        capturing_struct_id: u32,
        captured_columns: Vec<SemanticColumn>,
        select_table_ids: Vec<u32>,
        join_clauses: Vec<(SemanticColumn, SemanticColumn)>,
        where_clause: Option<SelectWhereClause>,
        limit_clause: Option<SelectCountClause>,
        offset_clause: Option<SelectCountClause>,
    },
    Insert {
        table_id: u32,
        value: Box<SemanticExpression>,
    },
    Update {
        table_id: u32,
        assignments: Vec<UpdateAssignment>,
        where_clause: Option<WhereClause>,
    },
    Delete {
        table_id: u32,
        where_clause: Option<WhereClause>,
    }
}

pub struct UpdateAssignment {
    pub column_index: u32,
    pub value: SemanticExpression,
}

pub struct WhereClause {
    pub column_index: u32,
    pub value: Box<SemanticExpression>,
}

pub struct SelectWhereClause {
    pub column: SemanticColumn,
    pub value: Box<SemanticExpression>,
}

pub struct SelectCountClause {
    pub value: Box<SemanticExpression>,
}

pub struct SemanticColumn {
    pub table_index: u32,
    pub column_index: u32,
}
