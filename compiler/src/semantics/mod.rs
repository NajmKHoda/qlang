mod types;
mod ir;
mod variables;
mod queries;
mod functions;
mod data;
mod binops;
mod errors;

use std::collections::HashMap;
use std::rc::Rc;

use types::*;
use variables::*;
use functions::*;
use ir::*;
use queries::*;
use errors::SemanticError;

use crate::tokens::*;

struct SemanticStruct {
    name: String,
    fields: HashMap<String, SemanticType>,
    field_order: Vec<String>,
}

struct SemanticGen {
    tables: HashMap<String, Rc<SemanticTable>>,
    structs: HashMap<String, Rc<SemanticStruct>>,
    functions: HashMap<String, Rc<SemanticFunction>>,
    variables: Vec<HashMap<String, Rc<SemanticVariable>>>,
}

impl SemanticGen {
    fn new() -> Self {
        SemanticGen {
            tables: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            variables: vec![],
        }
    }

    fn eval_stmt(&mut self, _: &StatementNode) -> Result<SemanticStatement, SemanticError> {
        unimplemented!()
    }

    fn eval_expr(&self, expr: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        match expr {
            ExpressionNode::IntegerLiteral(val) => {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::IntegerLiteral(*val),
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                })
            },
            ExpressionNode::BoolLiteral(val) => {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::BoolLiteral(*val),
                    sem_type: SemanticType::new(SemanticTypeKind::Bool),
                })
            },
            ExpressionNode::StringLiteral(val) => {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::StringLiteral(val.clone()),
                    sem_type: SemanticType::new(SemanticTypeKind::String),
                })
            },
            ExpressionNode::Struct(struct_name_opt, column_values) => {
                self.eval_struct(struct_name_opt.as_deref(), column_values)
            },
            ExpressionNode::Array(elements) => {
                self.eval_array(elements)
            },
            ExpressionNode::QName(qname) => {
                let variable = self.get_variable(qname)?;
                Ok(SemanticExpression { 
                    kind: SemanticExpressionKind::Variable(variable.clone()), 
                    sem_type: variable.sem_type.clone(),
                })
            },
            ExpressionNode::StructField(struct_expr, field_name) => {
                self.eval_struct_field(struct_expr, field_name)
            },
            ExpressionNode::ArrayIndex(array_expr, index_expr) => {
                self.eval_array_index(array_expr, index_expr)
            },
            ExpressionNode::Add(left, right) => {
                self.eval_add(left, right)
            }
            ExpressionNode::Subtract(left, right) => {
                self.eval_subtract(left, right)
            },
            ExpressionNode::Comparison(left, right, op) => {
                self.eval_compare(left, right, *op)
            }
            ExpressionNode::FunctionCall(func_name, args) => {
                self.call_function(func_name, args)
            }
            ExpressionNode::MethodCall(receiver, method_name, args) => {
                self.call_method(receiver, method_name, args)
            }
            ExpressionNode::Query(query_node) => {
                match query_node {
                    QueryNode::Select(select) => self.eval_select_query(select),
                    QueryNode::Insert(insert) => self.eval_insert_query(insert),
                    QueryNode::Update(update) => self.eval_update_query(update),
                    QueryNode::Delete(delete) => self.eval_delete_query(delete),
                }
            }
        }
    }
}
