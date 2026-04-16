mod types;
mod ir;
mod variables;
mod queries;
mod functions;
mod closures;
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
pub use closures::*;
pub use control_flow::*;
pub use data::*;
pub use ir::*;
pub use queries::*;
pub use errors::SemanticError;

use crate::tokens::*;

#[derive(Clone, Copy)]
pub enum Executable {
    Function(u32),
    Closure(u32),
}

pub struct SemanticGen {
    datasources: DualLookup<SemanticDatasource>,
    tables: DualLookup<SemanticTable>,
    structs: DualLookup<SemanticStruct>,
    functions: DualLookup<SemanticFunction>,
    closures: HashMap<u32, SemanticClosure>,
    variables: HashMap<u32, SemanticVariable>,
    scopes: Vec<SemanticScope>,
    loops: Vec<(Option<String>, u32)>,
    cur_function: Option<Executable>,

    datasource_id_gen: IdGenerator,
    table_id_gen: IdGenerator,
    struct_id_gen: IdGenerator,
    function_id_gen: IdGenerator,
    closure_id_gen: IdGenerator,
    variable_id_gen: IdGenerator,
    loop_id_gen: IdGenerator,
    transaction_id_gen: IdGenerator,
}
    
pub struct SemanticProgram {
    pub datasources: HashMap<u32, SemanticDatasource>,
    pub tables: HashMap<u32, SemanticTable>,
    pub structs: HashMap<u32, SemanticStruct>,
    pub functions: HashMap<u32, SemanticFunction>,
    pub closures: HashMap<u32, SemanticClosure>,
    pub variables: HashMap<u32, SemanticVariable>,
}

impl SemanticGen {
    fn new() -> Self {
        SemanticGen {
            datasources: DualLookup::new(),
            tables: DualLookup::new(),
            structs: DualLookup::new(),
            functions: DualLookup::new(),
            closures: HashMap::new(),
            variables: HashMap::new(),
            scopes: vec![],
            loops: vec![],
            cur_function: None,

            datasource_id_gen: IdGenerator::new(),
            table_id_gen: IdGenerator::new(),
            struct_id_gen: IdGenerator::new(),
            function_id_gen: IdGenerator::new(),
            closure_id_gen: IdGenerator::new(),
            variable_id_gen: IdGenerator::new(),
            loop_id_gen: IdGenerator::new(),
            transaction_id_gen: IdGenerator::new(),
        }
    }

    fn eval_stmt(&mut self, stmt: &StatementNode) -> Result<Vec<SemanticStatement>, SemanticError> {
        match stmt {
            StatementNode::VariableDefinition { var_type, name, init_expr } => {
                self.define_variable(name, var_type, init_expr).map(|s| vec![s])
            },
            StatementNode::Assignment { name, expr } => {
                self.assign_variable(name, expr).map(|s| vec![s])
            },
            StatementNode::LoneExpression(expr) => {
                let sem_expr = self.eval_expr(expr)?;
                Ok(vec![SemanticStatement::LoneExpression(sem_expr)])
            },
            StatementNode::Conditional { branches, else_branch } => {
                self.eval_conditional(branches, else_branch).map(|s| vec![s])
            },
            StatementNode::ConditionalLoop { condition, body, label } => {
                self.eval_conditional_loop(condition, body, label).map(|s| vec![s])
            },
            StatementNode::ForLoop { variable_name, iterable_expr, body, label } => {
                self.eval_for_loop(variable_name, iterable_expr, body, label)
            },
            StatementNode::Transaction { body, rollback_body } => {
                self.eval_transaction(body, rollback_body).map(|s| vec![s])
            }
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

    fn eval_expr(&mut self, expr: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
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
            ExpressionNode::Range { start, end, inclusive, step } => {
                let sem_start = match start {
                    Some(expr) => Some(Box::new(self.eval_expr(expr)?)),
                    None => None,
                };
                let sem_end = match end {
                    Some(expr) => Some(Box::new(self.eval_expr(expr)?)),
                    None => None,
                };
                let sem_step = match step {
                    Some(expr) => Some(Box::new(self.eval_expr(expr)?)),
                    None => None,
                };

                if *inclusive && sem_end.is_none() {
                    return Err(SemanticError::IncompatibleOperands {
                        operation: "range".to_string(),
                        left_type: SemanticType::new(SemanticTypeKind::Void),
                        right_type: SemanticType::new(SemanticTypeKind::Void),
                    });
                }

                let int_type = SemanticType::new(SemanticTypeKind::Integer);
                if let Some(ref expr) = sem_start {
                    if !self.try_downcast(&int_type, &expr.sem_type) {
                        return Err(SemanticError::IncompatibleOperands {
                            operation: "range".to_string(),
                            left_type: expr.sem_type.clone(),
                            right_type: int_type.clone(),
                        });
                    }
                }
                if let Some(ref expr) = sem_end {
                    if !self.try_downcast(&int_type, &expr.sem_type) {
                        return Err(SemanticError::IncompatibleOperands {
                            operation: "range".to_string(),
                            left_type: expr.sem_type.clone(),
                            right_type: int_type.clone(),
                        });
                    }
                }
                if let Some(ref expr) = sem_step {
                    if !self.try_downcast(&int_type, &expr.sem_type) {
                        return Err(SemanticError::IncompatibleOperands {
                            operation: "range".to_string(),
                            left_type: expr.sem_type.clone(),
                            right_type: int_type.clone(),
                        });
                    }
                }

                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::Range {
                        start: sem_start,
                        end: sem_end,
                        inclusive: *inclusive,
                        step: sem_step,
                    },
                    sem_type: SemanticType::new(SemanticTypeKind::Iterator(int_type.clone())),
                    ownership: Ownership::Owned,
                })
            }
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
            ExpressionNode::Closure { is_failable, params, return_type, body } => {
                self.eval_closure(*is_failable, params, return_type.as_ref(), body)
            }
            ExpressionNode::ImmediateQuery(query_node) => {
                self.eval_immediate_query(query_node)
            }
            ExpressionNode::ParameterizedQuery { parameters, query } => {
                self.eval_parameterized_query(parameters, query)
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
        for _struct in &program.structs {
            self.define_struct_type(&_struct.name, &_struct.fields)?;
        }

        for function in &program.functions {
            self.declare_function(&function.name, function.is_failable, &function.params, &function.return_type)?;
        }
        if !self.functions.contains_name("main") {
            return Err(SemanticError::MissingMainFunction);
        }

        for function in &program.functions {
            let func_id = self.functions[function.name.as_str()].id;
            self.define_function(func_id, &function.body)?;
        }

        Ok(SemanticProgram {
            datasources: self.datasources.collect_id_value_map(),
            tables: self.tables.collect_id_value_map(),
            structs: self.structs.collect_id_value_map(),
            functions: self.functions.collect_id_value_map(),
            closures: self.closures,
            variables: self.variables,
        })
    }

    pub fn gen_semantic(program: &ProgramNode) -> Result<SemanticProgram, SemanticError> {
        let sem_gen = SemanticGen::new();
        sem_gen.eval_program(program)
    }

    pub(super) fn cur_return_type(&self) -> SemanticType {
        match self.cur_function {
            Some(Executable::Function(id)) => self.functions[id].return_type.clone(),
            Some(Executable::Closure(id)) => self.closures[&id].return_type.clone(),
            None => SemanticType::new(SemanticTypeKind::Void),
        }
    }

    pub(super) fn cur_executable_name(&self) -> String {
        match self.cur_function {
            Some(Executable::Function(id)) => self.functions[id].name.clone(),
            Some(Executable::Closure(id)) => format!("<closure@{}>", id),
            None => "<global>".to_string(),
        }
    }

    pub(super) fn cur_function_is_failable(&self) -> bool {
        match self.cur_function {
            Some(Executable::Function(id)) => self.functions[id].is_failable,
            Some(Executable::Closure(id)) => self.closures[&id].is_failable,
            None => false,
        }
    }

    pub(super) fn cur_error_drops(&self) -> Vec<u32> {
        let mut drops = vec![];
        for scope in self.scopes.iter().rev() {
            match scope.scope_type {
                SemanticScopeType::Function | SemanticScopeType::Closure(_) => break,
                _ => {}
            }
            drops.extend(scope.variables.values().copied());
        }
        drops
    }
}
