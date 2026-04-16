use crate::{semantics::SemanticQuery, tokens::{ClosureBodyNode, TypeNode, TypedQNameNode}};

use super::{Executable, SemanticGen, SemanticType, SemanticBlock, SemanticScopeType, SemanticError, SemanticVariable, Ownership, SemanticExpression, SemanticExpressionKind, SemanticStatement, SemanticTypeKind};

pub struct SemanticClosure {
    pub id: u32,
    pub is_failable: bool,
    pub param_ids: Vec<u32>,
    pub captured_variables: Vec<(u32, u32)>,
    pub return_type: SemanticType,
    pub body: SemanticClosureBody,
}

pub enum SemanticClosureBody {
    Procedural(SemanticBlock),
    Query(SemanticQuery),
}

impl SemanticClosureBody {
    pub(super) fn dummy() -> Self {
        SemanticClosureBody::Procedural(SemanticBlock {
            statements: vec![],
            terminates: false,
        })
    }
}

impl SemanticGen {
    pub fn eval_closure(
        &mut self,
        is_failable: bool,
        parameter_nodes: &[TypedQNameNode],
        return_type: Option<&TypeNode>,
        body: &ClosureBodyNode
    ) -> Result<SemanticExpression, SemanticError> {
        let id = self.closure_id_gen.next_id();
        let mut param_ids: Vec<u32> = vec![];
        let mut sem_param_types: Vec<SemanticType> = vec![];

        // Create parameter variables at closure scope
        self.enter_scope(SemanticScopeType::Closure(id));
        for param_node in parameter_nodes {
            let sem_type = self.try_get_semantic_type(&param_node.type_node)?;
            let variable_id = self.variable_id_gen.next_id();
    
            self.scopes.last_mut().unwrap().variables.insert(param_node.name.clone(), variable_id);
            self.variables.insert(variable_id, SemanticVariable {
                name: param_node.name.clone(),
                id: variable_id,
                sem_type: sem_type.clone(),
            });
            param_ids.push(variable_id);
            sem_param_types.push(sem_type);
        }

        let sem_ret_type = match return_type {
            Some(ret_type_node) => self.try_get_semantic_type(ret_type_node)?,
            None => SemanticType::new(SemanticTypeKind::Any),
        };
        self.closures.insert(id, SemanticClosure {
            id,
            is_failable,
            param_ids,
            captured_variables: vec![],
            return_type: sem_ret_type.clone(),
            body: SemanticClosureBody::dummy()
        });

        let prev_executable = self.cur_function;
        self.cur_function = Some(Executable::Closure(id));

        match body {
            ClosureBodyNode::Expression(expr_node) => {
                let ret_expr = self.eval_expr(expr_node)?;
                if !self.try_unify(&sem_ret_type, &ret_expr.sem_type) {
                    return Err(SemanticError::MistypedReturnValue {
                        expected: sem_ret_type,
                        found: ret_expr.sem_type,
                    })
                }
                let return_var_id = self.variable_id_gen.next_id();
                self.variables.insert(return_var_id, SemanticVariable {
                    name: format!("__ql__ret_{}", return_var_id),
                    id: return_var_id,
                    sem_type: ret_expr.sem_type.clone(),
                });

                let closure = self.closures.get_mut(&id).unwrap();
                closure.body = SemanticClosureBody::Procedural(SemanticBlock {
                    statements: vec![
                        SemanticStatement::VariableDeclaration {
                            variable_id: return_var_id,
                            init_expr: ret_expr,
                        },
                        SemanticStatement::Return(Some(return_var_id))
                    ],
                    terminates: true
                });
            },
            ClosureBodyNode::Statements(stmts) => {
                self.enter_scope(SemanticScopeType::Block);
                let mut body_block = self.eval_block(stmts)?;

                if !body_block.terminates {
                    let void_type = SemanticType::new(SemanticTypeKind::Void);
                    if self.try_downcast(&void_type, &sem_ret_type) {
                        let return_stmt = SemanticStatement::Return(None);
                        body_block.statements.push(return_stmt);
                        body_block.terminates = true;
                    } else {
                        return Err(SemanticError::InexhaustiveReturnPaths {
                            function_name: format!("<closure@{}>", id),
                        });
                    }
                }

                let closure = self.closures.get_mut(&id).unwrap();
                closure.body = SemanticClosureBody::Procedural(body_block);
            },
        }
        self.cur_function = prev_executable;

        self.exit_scope(false);
        if !sem_ret_type.is_concrete() {
            return Err(SemanticError::AmbiguousReturnType {
                return_type: sem_ret_type,
            })
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Callable {
                is_failable,
                param_types: sem_param_types,
                ret_type: sem_ret_type,
            }),
            kind: SemanticExpressionKind::Closure(id),
            ownership: Ownership::Owned,
        })
    }
}