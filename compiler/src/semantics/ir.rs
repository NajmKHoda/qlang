#![allow(dead_code)]

use std::{collections::HashMap, rc::Rc};

use crate::semantics::control_flow::SemanticBlock;

use super::*;

pub enum SemanticStatement {
    VariableDeclaration {
        variable: Rc<SemanticVariable>,
        init_expr: SemanticExpression,
    },
    VariableAssignment {
        variable: Rc<SemanticVariable>,
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
        id: LoopId,
    },
    Return(Option<SemanticExpression>),
    Break(LoopId),
    Continue(LoopId),
    DropVariable(Rc<SemanticVariable>),
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
    Variable(Rc<SemanticVariable>),
    StructField {
        struct_expr: Box<SemanticExpression>,
        index: i32,
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
    FunctionCall {
        function: Rc<SemanticFunction>,
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

pub enum BuiltinFunction {
    PrintString,
    PrintInteger,
    PrintBool,
    InputInteger,
    InputString,
}

pub enum BuiltinMethod {
    ArrayLength,
    ArrayAppend,
    ArrayPop
}

pub enum SemanticQuery {
    Select {
        from_table: Rc<SemanticTable>,
        where_clause: Option<WhereClause>,
    },
    Insert {
        into_table: Rc<SemanticTable>,
        value: Box<SemanticExpression>,
    },
    Update {
        table: Rc<SemanticTable>,
        assignments: HashMap<String, SemanticExpression>,
        where_clause: Option<WhereClause>,
    },
    Delete {
        from_table: Rc<SemanticTable>,
        where_clause: Option<WhereClause>,
    }
}

pub struct WhereClause {
    pub(super) column_name: String,
    pub(super) value: Box<SemanticExpression>,
}
