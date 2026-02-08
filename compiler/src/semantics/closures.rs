use crate::tokens::{ClosureBodyNode, TypeNode, TypedQNameNode};

use super::{SemanticGen, SemanticType, SemanticBlock, SemanticParameter, SemanticScopeType, SemanticError, SemanticVariable, Ownership, SemanticExpression, SemanticExpressionKind, SemanticStatement, SemanticTypeKind};

pub struct SemanticClosure {
    pub id: u32,
    pub parameters: Vec<SemanticParameter>,
    pub captured_variables: Vec<(u32, u32)>,
    pub return_type: SemanticType,
    pub body: SemanticBlock,
}

impl SemanticGen {
    pub fn eval_closure(
        &mut self,
        parameter_nodes: &[TypedQNameNode],
        return_type: Option<&TypeNode>,
        body: &ClosureBodyNode
    ) -> Result<SemanticExpression, SemanticError> {
        let id = self.closure_id_gen.next_id();
        let mut params: Vec<SemanticParameter> = vec![];
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
            params.push(SemanticParameter {
                name: param_node.name.clone(),
                variable_id,
                sem_type: sem_type.clone(),
            });
            sem_param_types.push(sem_type);
        }

        let sem_ret_type = match return_type {
            Some(ret_type_node) => self.try_get_semantic_type(ret_type_node)?,
            None => SemanticType::new(SemanticTypeKind::Any),
        };
        self.closures.insert(id, SemanticClosure {
            id,
            parameters: params,
            captured_variables: vec![],
            return_type: sem_ret_type.clone(),
            body: SemanticBlock {
                statements: vec![],
                terminates: false,
            },
        });

        match body {
            ClosureBodyNode::Expression(expr_node) => {
                let ret_expr = self.eval_expr(expr_node)?;
                if !self.try_unify(&sem_ret_type, &ret_expr.sem_type) {
                    return Err(SemanticError::MistypedReturnValue {
                        expected: sem_ret_type,
                        found: ret_expr.sem_type,
                    })
                }
                self.closures.get_mut(&id).unwrap().body = SemanticBlock {
                    statements: vec![SemanticStatement::Return(Some(ret_expr))],
                    terminates: true
                }
            },
            ClosureBodyNode::Statements(stmts) => {
                let prev_return_type = self.cur_return_type.clone();
                self.cur_return_type = sem_ret_type.clone();
                let mut body_block = self.eval_block(stmts, SemanticScopeType::Block)?;

                self.exit_scope(false);
                self.cur_return_type = prev_return_type;

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

                self.closures.get_mut(&id).unwrap().body = body_block;
            },
        }

        if !sem_ret_type.is_concrete() {
            return Err(SemanticError::AmbiguousReturnType {
                return_type: sem_ret_type,
            })
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Callable(sem_param_types, sem_ret_type)),
            kind: SemanticExpressionKind::Closure(id),
            ownership: Ownership::Owned,
        })
    }
}