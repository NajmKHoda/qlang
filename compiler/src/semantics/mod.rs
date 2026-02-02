mod types;
mod ir;
mod variables;
mod queries;
mod functions;
mod control_flow;
mod data;
mod binops;
mod errors;
mod util;

use std::{collections::HashMap};
use util::*;

pub use types::*;
pub use variables::*;
pub use functions::*;
pub use control_flow::*;
pub use data::*;
pub use ir::*;
pub use queries::*;
pub use errors::SemanticError;

use crate::tokens::*;

pub struct SemanticGen {
    datasources: DualLookup<SemanticDatasource>,
    tables: DualLookup<SemanticTable>,
    structs: DualLookup<SemanticStruct>,
    functions: DualLookup<SemanticFunction>,
    variables: HashMap<u32, SemanticVariable>,
    scopes: Vec<SemanticScope>,
    loops: Vec<(Option<String>, u32)>,
    cur_return_type: SemanticType,

    datasource_id_gen: IdGenerator,
    table_id_gen: IdGenerator,
    struct_id_gen: IdGenerator,
    function_id_gen: IdGenerator,
    variable_id_gen: IdGenerator,
    loop_id_gen: IdGenerator,
}
    
pub struct SemanticProgram {
    pub datasources: HashMap<u32, SemanticDatasource>,
    pub tables: HashMap<u32, SemanticTable>,
    pub structs: HashMap<u32, SemanticStruct>,
    pub functions: HashMap<u32, SemanticFunction>,
    pub variables: HashMap<u32, SemanticVariable>,
}

impl SemanticGen {
    fn new() -> Self {
        SemanticGen {
            datasources: DualLookup::new(),
            tables: DualLookup::new(),
            structs: DualLookup::new(),
            functions: DualLookup::new(),
            variables: HashMap::new(),
            scopes: vec![],
            loops: vec![],
            cur_return_type: SemanticType::new(SemanticTypeKind::Void),

            datasource_id_gen: IdGenerator::new(),
            table_id_gen: IdGenerator::new(),
            struct_id_gen: IdGenerator::new(),
            function_id_gen: IdGenerator::new(),
            variable_id_gen: IdGenerator::new(),
            loop_id_gen: IdGenerator::new(),
        }
    }

    fn eval_stmt(&mut self, stmt: &StatementNode) -> Result<Vec<SemanticStatement>, SemanticError> {
        match stmt {
            StatementNode::VariableDefinition(var_type, name, init_expr) => {
                self.define_variable(name, var_type, init_expr).map(|s| vec![s])
            },
            StatementNode::Assignment(var_name, expr) => {
                self.assign_variable(var_name, expr).map(|s| vec![s])
            },
            StatementNode::LoneExpression(expr) => {
                let sem_expr = self.eval_expr(expr)?;
                Ok(vec![SemanticStatement::LoneExpression(sem_expr)])
            },
            StatementNode::Conditional(branches, else_branch) => {
                self.eval_conditional(branches, else_branch).map(|s| vec![s])
            },
            StatementNode::ConditionalLoop(condition, body, label) => {
                self.eval_conditional_loop(condition, body, label).map(|s| vec![s])
            },
            StatementNode::Return(expr) => {
                self.eval_return(expr.as_deref())
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
                    ownership: Ownership::Trivial,
                })
            },
            ExpressionNode::BoolLiteral(val) => {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::BoolLiteral(*val),
                    sem_type: SemanticType::new(SemanticTypeKind::Bool),
                    ownership: Ownership::Trivial,
                })
            },
            ExpressionNode::StringLiteral(val) => {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::StringLiteral(val.clone()),
                    sem_type: SemanticType::new(SemanticTypeKind::String),
                    ownership: Ownership::Borrowed,
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
                    kind: SemanticExpressionKind::Variable(variable.id), 
                    sem_type: variable.sem_type.clone(),
                    ownership: if variable.sem_type.can_be_owned() {
                        Ownership::Borrowed
                    } else {
                        Ownership::Trivial
                    },
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

    pub fn eval_program(mut self, program: &ProgramNode) -> Result<SemanticProgram, SemanticError> {
        for datasource in &program.datasources {
            self.declare_datasource(&datasource.name, datasource.is_readonly)?;
        }

        for table in &program.tables {
            self.define_table(&table.name, &table.columns, table.is_readonly, &table.datasource_name)?;
        }

        for function in &program.functions {
            self.define_function(&function.name, &function.params, &function.return_type, &function.body)?;
        }

        Ok(SemanticProgram {
            datasources: self.datasources.collect_id_value_map(),
            tables: self.tables.collect_id_value_map(),
            structs: self.structs.collect_id_value_map(),
            functions: self.functions.collect_id_value_map(),
            variables: self.variables,
        })
    }

    pub fn gen_semantic(program: &ProgramNode) -> Result<SemanticProgram, SemanticError> {
        let sem_gen = SemanticGen::new();
        sem_gen.eval_program(program)
    }
}
