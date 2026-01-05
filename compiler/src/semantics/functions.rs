use std::rc::Rc;

use super::*;

pub(super) struct SemanticFunction {
    pub(super) params: Vec<Rc<SemanticVariable>>,
    pub(super) return_type: SemanticType,
    pub(super) body: SemanticBlock,
}

impl SemanticGen {
    pub(super) fn define_function(
        &mut self,
        name: &str,
        param_nodes: &[TypedQNameNode],
        return_type: &TypeNode,
        body: &[StatementNode],
    ) -> Result<(), SemanticError> {
        if self.functions.contains_key(name) {
            return Err(SemanticError::DuplicateFunctionDefinition {
                name: name.to_string(),
            });
        }

        self.cur_return_type = self.try_get_semantic_type(return_type)?;
        self.variables.push(HashMap::new());
        let mut params = Vec::new();
        for param_node in param_nodes {
            let param_type = self.try_get_semantic_type(&param_node.type_node)?;
            let variable = Rc::new(SemanticVariable { sem_type: param_type.clone() });

            let current_scope = self.variables.last_mut().unwrap();
            current_scope.insert(param_node.name.clone(), variable.clone());
            params.push(variable);
        }

        if name == "main" && (!params.is_empty() || self.cur_return_type != SemanticTypeKind::Integer) {
            return Err(SemanticError::InvalidMainSignature);
        }

        self.variables.push(HashMap::new());
        let mut body_block = self.eval_block(body)?;
        self.variables.pop();
        self.variables.pop();

        if !body_block.terminates {
            if self.cur_return_type == SemanticTypeKind::Void {
                body_block.statements.push(SemanticStatement::Return(None));
            } else if name == "main" {
                let zero_expr = SemanticExpression {
                    kind: SemanticExpressionKind::IntegerLiteral(0),
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                };
                body_block.statements.push(SemanticStatement::Return(Some(zero_expr)));
            } else {
                return Err(SemanticError::InexhaustiveReturnPaths {
                    function_name: name.to_string(),
                });
            }
        }

        self.functions.insert(name.to_string(), Rc::new(SemanticFunction {
            params,
            return_type: self.cur_return_type.clone(),
            body: body_block,
        }));
        Ok(())
    }

    pub(super) fn call_function(&self, name: &str, arg_exprs: &[Box<ExpressionNode>]) -> Result<SemanticExpression, SemanticError> {
        if let Some(func) = self.functions.get(name) {
            if arg_exprs.len() != func.params.len() {
                return Err(SemanticError::MismatchingCallArity {
                    function_name: name.to_string(),
                    expected: func.params.len(),
                    found: arg_exprs.len(),
                });
            }

            let sem_args = arg_exprs.iter()
                .map(|arg| self.eval_expr(arg))
                .collect::<Result<Vec<SemanticExpression>, SemanticError>>()?;
            for (i, (arg, param)) in sem_args.iter().zip(&func.params).enumerate() {
                let compatible = arg.sem_type.try_downcast(&param.sem_type);
                if !compatible {
                    return Err(SemanticError::IncompatibleArgumentType {
                        function_name: name.to_string(),
                        position: i,
                        expected: param.sem_type.clone(),
                        found: arg.sem_type.clone(),
                    });
                }
            }

            Ok(SemanticExpression {
                sem_type: func.return_type.clone(),
                kind: SemanticExpressionKind::FunctionCall {
                    function: func.clone(),
                    args: sem_args,
                },
            })
        } else {
            Err(SemanticError::UndefinedFunction { name: name.to_string() })
        }
    }

    pub(super) fn call_method(
        &self,
        receiver: &ExpressionNode,
        method_name: &str,
        arg_exprs: &[Box<ExpressionNode>]
    ) -> Result<SemanticExpression, SemanticError> {
        let sem_receiver = self.eval_expr(receiver)?;
        let sem_args = arg_exprs.iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<Vec<SemanticExpression>, SemanticError>>()?;

        let receiver_type = &sem_receiver.sem_type;
        match (receiver_type.kind(), method_name) {
            (SemanticTypeKind::Array(_), "length") => {
                if !sem_args.is_empty() {
                    return Err(SemanticError::MismatchingCallArity {
                        function_name: format!("{}.length", receiver_type),
                        expected: 0,
                        found: sem_args.len()
                    });
                }
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayLength,
                        args: vec![]
                    }
                })
            }
            (SemanticTypeKind::Array(elem_type), "append") => {
                if sem_args.len() != 1 {
                    return Err(SemanticError::MismatchingCallArity {
                        function_name: format!("{}.append", receiver_type),
                        expected: 1,
                        found: sem_args.len()
                    });
                }
                if !sem_args[0].sem_type.try_downcast(&elem_type) {
                    return Err(SemanticError::IncompatibleArgumentType {
                        function_name: format!("{}.append", receiver_type),
                        position: 0,
                        expected: elem_type,
                        found: sem_args[0].sem_type.clone(),
                    });
                }
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Void),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayAppend,
                        args: sem_args
                    }
                })
            }
            (SemanticTypeKind::Array(elem_type), "pop") => {
                if !sem_args.is_empty() {
                    return Err(SemanticError::MismatchingCallArity {
                        function_name: format!("{}.pop", receiver_type),
                        expected: 0,
                        found: sem_args.len()
                    });
                }
                Ok(SemanticExpression {
                    sem_type: elem_type,
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayPop,
                        args: vec![]
                    }
                })
            }
            _ => {
                Err(SemanticError::UndefinedMethod {
                    receiver_type: sem_receiver.sem_type,
                    method_name: method_name.to_string(),
                })
            }
        }
    }
}