mod types;
mod ir;
mod variables;
mod queries;
mod functions;
mod control_flow;
mod data;
mod binops;
mod errors;

use std::collections::HashMap;
use std::rc::Rc;

use types::*;
use variables::*;
use functions::*;
use control_flow::*;
use data::*;
use ir::*;
use queries::*;
use errors::SemanticError;

use crate::tokens::*;

pub struct SemanticGen {
    datasources: HashMap<String, Rc<SemanticDatasource>>,
    tables: HashMap<String, Rc<SemanticTable>>,
    structs: HashMap<String, Rc<SemanticStruct>>,
    functions: HashMap<String, Rc<SemanticFunction>>,
    variables: Vec<HashMap<String, Rc<SemanticVariable>>>,
    loops: Vec<(Option<String>, LoopId)>,

    cur_return_type: SemanticType,
    _next_loop_id: usize,
}

pub struct SemanticProgram {
    pub datasources: Vec<Rc<SemanticDatasource>>,
    pub tables: Vec<Rc<SemanticTable>>,
    pub structs: Vec<Rc<SemanticStruct>>,
    pub functions: Vec<Rc<SemanticFunction>>,
}

impl SemanticGen {
    fn new() -> Self {
        SemanticGen {
            datasources: HashMap::new(),
            tables: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            variables: vec![],
            loops: vec![],
            cur_return_type: SemanticType::new(SemanticTypeKind::Void),
            _next_loop_id: 0,
        }
    }

    fn eval_stmt(&mut self, stmt: &StatementNode) -> Result<SemanticStatement, SemanticError> {
        match stmt {
            StatementNode::VariableDefinition(var_type, name, init_expr) => {
                self.define_variable(name, var_type, init_expr)
            },
            StatementNode::Assignment(var_name, expr) => {
                self.assign_variable(var_name, expr)
            },
            StatementNode::LoneExpression(expr) => {
                let sem_expr = self.eval_expr(expr)?;
                Ok(SemanticStatement::LoneExpression(sem_expr))
            },
            StatementNode::Conditional(branches, else_branch) => {
                self.eval_conditional(branches, else_branch)
            },
            StatementNode::ConditionalLoop(condition, body, label) => {
                self.eval_conditional_loop(condition, body, label)
            },
            StatementNode::Return(expr) => {
                self.eval_return(expr)
            },
            StatementNode::Break(label) => {
                self.eval_break(label)
            },
            StatementNode::Continue(label) => {
                self.eval_continue(label)
            },
        }
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

    pub fn eval_program(&mut self, program: &ProgramNode) -> Result<SemanticProgram, SemanticError> {
        for datasource in &program.datasources {
            self.declare_datasource(&datasource.name, datasource.is_readonly)?;
        }

        for table in &program.tables {
            self.define_table(&table.name, &table.columns, &table.datasource_name)?;
        }

        for function in &program.functions {
            self.define_function(&function.name, &function.params, &function.return_type, &function.body)?;
        }

        Ok(SemanticProgram {
            datasources: self.datasources.values().cloned().collect(),
            tables: self.tables.values().cloned().collect(),
            structs: self.structs.values().cloned().collect(),
            functions: self.functions.values().cloned().collect(),
        })
    }

    pub fn gen_semantic(program: &ProgramNode) -> Result<SemanticProgram, SemanticError> {
        let mut sem_gen = SemanticGen::new();
        sem_gen.eval_program(program)
    }
}
