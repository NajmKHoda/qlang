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
    Return(Option<SemanticExpression>),
    Break(u32),
    Continue(u32),
    DropVariable(u32),
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
    BoolLiteral(bool),
    StringLiteral(String),
    Struct(HashMap<String, SemanticExpression>),
    Array(Vec<SemanticExpression>),
    Closure(u32),
    Variable(u32),
    StructField {
        struct_expr: Box<SemanticExpression>,
        index: u32,
    },
    ArrayIndex {
        array_expr: Box<SemanticExpression>,
        index_expr: Box<SemanticExpression>,
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
    },
    IndirectFunctionCall {
        function_expr: Box<SemanticExpression>,
        args: Vec<SemanticExpression>,
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
    ImmediateQuery(SemanticQuery),
}

#[derive(Clone, Copy)]
pub enum BuiltinFunction {
    PrintString,
    PrintInteger,
    PrintBool,
    InputInteger,
    InputString,
}

#[derive(Clone, Copy)]
pub enum BuiltinMethod {
    ArrayLength,
    ArrayAppend,
    ArrayPop
}

pub enum SemanticQuery {
    Select {
        table_id: u32,
        where_clause: Option<WhereClause>,
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
