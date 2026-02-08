use super::*;

pub struct SemanticParameter {
    pub name: String,
    pub sem_type: SemanticType,
    pub variable_id: u32,
}

pub struct SemanticFunction {
    pub name: String,
    pub id: u32,
    pub params: Vec<SemanticParameter>,
    pub return_type: SemanticType,
    pub body: SemanticBlock,
}

const BUILTIN_FNS: &[&str] = &[
    "prints",
    "printi",
    "printb",
    "inputs",
    "inputi",
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
            _ => Err(SemanticError::UndefinedFunction { name: name.to_string() }),
        }
    }

    pub(super) fn declare_function(
        &mut self,
        name: &str,
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

        let params: Vec<SemanticParameter> = param_nodes.iter().map(|param_node| {
            let param_type = self.try_get_semantic_type(&param_node.type_node)?;
            let param_id = self.variable_id_gen.next_id();
            Ok(SemanticParameter {
                name: param_node.name.clone(),
                sem_type: param_type,
                variable_id: param_id,
            })
        }).collect::<Result<_, SemanticError>>()?;

        if name == "main" && (!params.is_empty() || sem_return_type != SemanticTypeKind::Integer) {
            return Err(SemanticError::InvalidMainSignature);
        }

        self.functions.insert(name.to_string(), function_id, SemanticFunction {
            name: name.to_string(),
            id: function_id,
            params,
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
        let parameter_scope = &mut self.scopes.last_mut().unwrap().variables;
        for param in self.functions[id].params.iter() {
            // Create associated variable
            parameter_scope.insert(param.name.clone(), param.variable_id);
            self.variables.insert(param.variable_id, SemanticVariable {
                name: param.name.clone(),
                sem_type: param.sem_type.clone(),
                id: param.variable_id,
            });
        }

        // Evaluate function body
        self.cur_return_type = self.functions[id].return_type.clone();
        let mut body_block = self.eval_block(body, SemanticScopeType::Block)?;
        if !body_block.terminates {
            if self.cur_return_type == SemanticTypeKind::Void {
                let ret_stmt = SemanticStatement::Return(None);
                body_block.statements.push(ret_stmt);
            } else if self.functions[id].name == "main" {
                let literal_zero = SemanticExpression {
                    kind: SemanticExpressionKind::IntegerLiteral(0),
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                    ownership: Ownership::Trivial,
                };
                let ret_stmt = SemanticStatement::Return(Some(literal_zero));
                body_block.statements.push(ret_stmt);
            } else {
                return Err(SemanticError::InexhaustiveReturnPaths {
                    function_name: self.functions[id].name.clone(),
                });
            }
        }
        self.functions[id].body = body_block;

        Ok(())
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
            if let SemanticTypeKind::Callable(param_types, ret_type) = var_type.kind() {
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
            let param_types: Vec<SemanticType> = func.params.iter()
                .map(|param| param.sem_type.clone())
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
            _ => {
                Err(SemanticError::UndefinedMethod {
                    receiver_type: sem_receiver.sem_type,
                    method_name: method_name.to_string(),
                })
            }
        }
    }
}