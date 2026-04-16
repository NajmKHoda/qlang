use super::*;

pub struct SemanticFunction {
    pub name: String,
    pub id: u32,
    pub is_failable: bool,
    pub param_ids: Vec<u32>,
    pub return_type: SemanticType,
    pub body: SemanticBlock,
}

const BUILTIN_FNS: &[&str] = &[
    "prints",
    "printi",
    "printb",
    "inputs",
    "inputi",
    "zip",
    "concat",
];

impl SemanticGen {
    fn check_args(
        &self,
        fn_name: &str,
        arg_exprs: &[SemanticExpression],
        param_types: &[SemanticType]
    ) -> Result<(), SemanticError> {
        if arg_exprs.len() != param_types.len() {
            return Err(SemanticError::MismatchingCallArity {
                function_name: fn_name.to_string(),
                expected: param_types.len(),
                found: arg_exprs.len(),
            });
        }

        for (i, (arg, param_type)) in arg_exprs.iter().zip(param_types).enumerate() {
            let compatible = self.try_downcast(param_type, &arg.sem_type);
            if !compatible {
                return Err(SemanticError::IncompatibleArgumentType {
                    function_name: fn_name.to_string(),
                    position: i,
                    expected: (*param_type).clone(),
                    found: arg.sem_type.clone(),
                });
            }
        }
        Ok(())
    }

    fn call_builtin_function(&self, name: &str, arg_exprs: Vec<SemanticExpression>) -> Result<SemanticExpression, SemanticError> {
        match name {
            "prints" => {
                self.check_args("prints", &arg_exprs, &[SemanticType::new(SemanticTypeKind::String)])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Void),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::PrintString,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Trivial,
                })
            }
            "printi" => {
                self.check_args("printi", &arg_exprs, &[SemanticType::new(SemanticTypeKind::Integer)])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Void),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::PrintInteger,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Trivial,
                })
            }
            "printb" => {
                self.check_args("printb", &arg_exprs, &[SemanticType::new(SemanticTypeKind::Bool)])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Void),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::PrintBool,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Trivial,
                })
            }
            "inputs" => {
                self.check_args("inputs", &arg_exprs, &[])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::String),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::InputString,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Owned,
                })
            }
            "inputi" => {
                self.check_args("inputi", &arg_exprs, &[])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::InputInteger,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Trivial,
                })
            }
            "zip" => {
                if arg_exprs.len() != 2 {
                    return Err(SemanticError::MismatchingCallArity {
                        function_name: "zip".to_string(),
                        expected: 2,
                        found: arg_exprs.len(),
                    });
                }

                let iter_any = SemanticType::new(SemanticTypeKind::Iterator(
                    SemanticType::new(SemanticTypeKind::Any)
                ));

                let iter_a_type = match arg_exprs[0].sem_type.kind() {
                    SemanticTypeKind::Iterator(elem_type) => elem_type,
                    _ => {
                        return Err(SemanticError::IncompatibleArgumentType {
                            function_name: "zip".to_string(),
                            position: 0,
                            expected: iter_any.clone(),
                            found: arg_exprs[0].sem_type.clone(),
                        });
                    }
                };

                let iter_b_type = match arg_exprs[1].sem_type.kind() {
                    SemanticTypeKind::Iterator(elem_type) => elem_type,
                    _ => {
                        return Err(SemanticError::IncompatibleArgumentType {
                            function_name: "zip".to_string(),
                            position: 1,
                            expected: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                            found: arg_exprs[1].sem_type.clone(),
                        });
                    }
                };

                if !self.try_unify(&iter_a_type, &iter_b_type) {
                    return Err(SemanticError::IncompatibleArgumentType {
                        function_name: "zip".to_string(),
                        position: 1,
                        expected: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                        found: arg_exprs[1].sem_type.clone(),
                    });
                }

                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::Zip,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Owned,
                })
            }
            "concat" => {
                if arg_exprs.len() != 2 {
                    return Err(SemanticError::MismatchingCallArity {
                        function_name: "concat".to_string(),
                        expected: 2,
                        found: arg_exprs.len(),
                    });
                }

                let iter_any = SemanticType::new(SemanticTypeKind::Iterator(
                    SemanticType::new(SemanticTypeKind::Any)
                ));

                let iter_a_type = match arg_exprs[0].sem_type.kind() {
                    SemanticTypeKind::Iterator(elem_type) => elem_type,
                    _ => {
                        return Err(SemanticError::IncompatibleArgumentType {
                            function_name: "concat".to_string(),
                            position: 0,
                            expected: iter_any.clone(),
                            found: arg_exprs[0].sem_type.clone(),
                        });
                    }
                };

                let iter_b_type = match arg_exprs[1].sem_type.kind() {
                    SemanticTypeKind::Iterator(elem_type) => elem_type,
                    _ => {
                        return Err(SemanticError::IncompatibleArgumentType {
                            function_name: "concat".to_string(),
                            position: 1,
                            expected: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                            found: arg_exprs[1].sem_type.clone(),
                        });
                    }
                };

                if !self.try_unify(&iter_a_type, &iter_b_type) {
                    return Err(SemanticError::IncompatibleArgumentType {
                        function_name: "concat".to_string(),
                        position: 1,
                        expected: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                        found: arg_exprs[1].sem_type.clone(),
                    });
                }

                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Iterator(iter_a_type.clone())),
                    kind: SemanticExpressionKind::BuiltinFunctionCall {
                        function: BuiltinFunction::Concat,
                        args: arg_exprs,
                    },
                    ownership: Ownership::Owned,
                })
            }
            _ => Err(SemanticError::UndefinedFunction { name: name.to_string() }),
        }
    }

    pub(super) fn declare_function(
        &mut self,
        name: &str,
        is_failable: bool,
        param_nodes: &[TypedQNameNode],
        return_type: &TypeNode,
    ) -> Result<(), SemanticError> {
        if self.functions.contains_name(name) {
            return Err(SemanticError::DuplicateFunctionDefinition {
                name: name.to_string(),
            });
        }

        let sem_return_type = self.try_get_semantic_type(return_type)?;
        let function_id = self.function_id_gen.next_id();
        let param_ids = self.eval_params(param_nodes)?;

        if name == "main" && (!param_ids.is_empty() || sem_return_type != SemanticTypeKind::Integer) {
            return Err(SemanticError::InvalidMainSignature);
        }

        self.functions.insert(name.to_string(), function_id, SemanticFunction {
            name: name.to_string(),
            id: function_id,
            is_failable,
            param_ids,
            return_type: sem_return_type,
            body: SemanticBlock {
                statements: vec![],
                terminates: false,
            },
        });

        Ok(())
    }

    pub(super) fn define_function(&mut self, id: u32, body: &[StatementNode]) -> Result<(), SemanticError> {
        // Set up function scope and parameters
        self.enter_scope(SemanticScopeType::Function);
        for &param_id in &self.functions[id].param_ids {
            let variable = &self.variables[&param_id];
            let scope = self.scopes.last_mut().unwrap();
            scope.variables.insert(variable.name.clone(), variable.id);
        }

        // Evaluate function body
        self.cur_function = Some(Executable::Function(id));
        self.enter_scope(SemanticScopeType::Function);
        let mut body_block = self.eval_block(body)?;
        if !body_block.terminates {
            if self.cur_return_type() == SemanticTypeKind::Void {
                let ret_stmt = SemanticStatement::Return(None);
                body_block.statements.push(ret_stmt);
            } else if self.functions[id].name == "main" {
                let return_var_id = self.variable_id_gen.next_id();
                self.variables.insert(return_var_id, SemanticVariable {
                    name: format!("__ql__ret_{}", return_var_id),
                    id: return_var_id,
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                });
                body_block.statements.push(SemanticStatement::VariableDeclaration {
                    variable_id: return_var_id,
                    init_expr: SemanticExpression {
                        kind: SemanticExpressionKind::IntegerLiteral(0),
                        sem_type: SemanticType::new(SemanticTypeKind::Integer),
                        ownership: Ownership::Trivial,
                    },
                });
                body_block.statements.push(SemanticStatement::Return(Some(return_var_id)));
            } else {
                return Err(SemanticError::InexhaustiveReturnPaths {
                    function_name: self.functions[id].name.clone(),
                });
            }
        }
        self.functions[id].body = body_block;

        Ok(())
    }

    pub(super) fn eval_params(&mut self, param_nodes: &[TypedQNameNode]) -> Result<Vec<u32>, SemanticError> {
        let mut param_ids = vec![];
        for param_node in param_nodes {
            let param_type = self.try_get_semantic_type(&param_node.type_node)?;
            if param_type == SemanticTypeKind::Void {
                return Err(SemanticError::VoidParameterType {
                    function_name: "<closure>".to_string(),
                    param_name: param_node.name.clone(),
                });
            }
            let var_id = self.variable_id_gen.next_id();
            self.variables.insert(var_id, SemanticVariable {
                name: param_node.name.clone(),
                sem_type: param_type,
                id: var_id,
            });
            param_ids.push(var_id);
        }
        Ok(param_ids)
    }

    pub(super) fn call_function(&mut self, name: &str, arg_exprs: &[Box<ExpressionNode>]) -> Result<SemanticExpression, SemanticError> {
        let sem_args = arg_exprs.iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<Vec<SemanticExpression>, SemanticError>>()?;
        if BUILTIN_FNS.contains(&name) {
            return self.call_builtin_function(name, sem_args);
        }

        if let Some(var) = self.get_variable_opt(name) {
            let var_id = var.id;
            let var_type = &var.sem_type.clone();
            if let SemanticTypeKind::Callable { is_failable, param_types, ret_type } = var_type.kind() {
                if is_failable && !self.cur_function_is_failable() {
                    return Err(SemanticError::FailableCallInNonFailableFunction {
                        caller_name: self.cur_executable_name(),
                        callee_name: name.to_string(),
                    });
                }
                self.check_args(name, &sem_args, &param_types)?;
                let expr_kind = SemanticExpressionKind::IndirectFunctionCall {
                    function_expr: Box::new(SemanticExpression {
                        kind: SemanticExpressionKind::Variable(var_id),
                        sem_type: var_type.clone(),
                        ownership: Ownership::Borrowed,
                    }),
                    args: sem_args,
                };
                return Ok(SemanticExpression {
                    sem_type: ret_type.clone(),
                    kind: expr_kind,
                    ownership: if ret_type.can_be_owned() {
                        Ownership::Owned
                    } else {
                        Ownership::Trivial
                    },
                });
            } else {
                Err(SemanticError::NotCallableType {
                    found_type: var_type.clone(),
                })
            }
        } else if let Some(func) = self.functions.get_by_name(name) {
            if func.is_failable && !self.cur_function_is_failable() {
                return Err(SemanticError::FailableCallInNonFailableFunction {
                    caller_name: self.cur_executable_name(),
                    callee_name: func.name.clone(),
                });
            }

            let param_types: Vec<SemanticType> = func.param_ids.iter()
                .map(|&param_id| self.variables[&param_id].sem_type.clone())
                .collect();
            self.check_args(name, &sem_args, &param_types)?;
            Ok(SemanticExpression {
                sem_type: func.return_type.clone(),
                kind: SemanticExpressionKind::DirectFunctionCall {
                    function_id: func.id,
                    args: sem_args,
                },
                ownership: if func.return_type.can_be_owned() {
                    Ownership::Owned
                } else {
                    Ownership::Trivial
                },
            })
        } else {
            Err(SemanticError::UndefinedFunction {
                name: name.to_string()
            })
        }
    }

    pub(super) fn call_method(
        &mut self,
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
                self.check_args("Array.length", &sem_args, &[])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayLength,
                        args: vec![]
                    },
                    ownership: Ownership::Trivial,
                })
            }
            (SemanticTypeKind::Array(elem_type), "append") => {
                self.check_args("Array.append", &sem_args, &[elem_type])?;
                Ok(SemanticExpression {
                    sem_type: SemanticType::new(SemanticTypeKind::Void),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayAppend,
                        args: sem_args
                    },
                    ownership: Ownership::Trivial,
                })
            }
            (SemanticTypeKind::Array(elem_type), "pop") => {
                self.check_args("Array.pop", &sem_args, &[])?;
                Ok(SemanticExpression {
                    ownership: if elem_type.can_be_owned() {
                        Ownership::Owned
                    } else {
                        Ownership::Trivial
                    },
                    sem_type: elem_type.clone(),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayPop,
                        args: vec![]
                    },
                })
            }
            (SemanticTypeKind::Array(elem_type), "iter") => {
                self.check_args("Array.iter", &sem_args, &[])?;
                Ok(SemanticExpression {
                    ownership: Ownership::Owned,
                    sem_type: SemanticType::new(SemanticTypeKind::Iterator(elem_type.clone())),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::ArrayIter,
                        args: vec![]
                    },
                })
            }
             (SemanticTypeKind::Iterator(elem_type), "next") => {
                self.check_args("Iterator.next", &sem_args, &[])?;
                Ok(SemanticExpression {
                    ownership: if elem_type.can_be_owned() {
                        Ownership::Borrowed
                    } else {
                        Ownership::Trivial
                    },
                    sem_type: elem_type.clone(),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::IteratorNext,
                        args: vec![]
                    },
                })
            }
            (SemanticTypeKind::Iterator(_), "has_next") => {
                self.check_args("Iterator.has_next", &sem_args, &[])?;
                Ok(SemanticExpression {
                    ownership: Ownership::Trivial,
                    sem_type: SemanticType::new(SemanticTypeKind::Bool),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::IteratorHasNext,
                        args: vec![]
                    },
                })
            }
            (SemanticTypeKind::Iterator(elem_type), "collect") => {
                self.check_args("Iterator.collect", &sem_args, &[])?;
                Ok(SemanticExpression {
                    ownership: Ownership::Owned,
                    sem_type: SemanticType::new(SemanticTypeKind::Array(elem_type.clone())),
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_receiver),
                        method: BuiltinMethod::IteratorCollect,
                        args: vec![]
                    },
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