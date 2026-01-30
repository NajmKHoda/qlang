use std::rc::Rc;

use super::*;

pub struct SemanticFunction {
    pub name: String,
    pub params: Vec<Rc<SemanticVariable>>,
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
            let compatible = arg.sem_type.try_downcast(param_type);
            if !compatible {
                return Err(SemanticError::IncompatibleArgumentType {
                    function_name: fn_name.to_string(),
                    position: i,
                    expected: param_type.clone(),
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

        let mut body_block = self.eval_block(body)?;
        self.variables.pop();

        if !body_block.terminates {
            if self.cur_return_type == SemanticTypeKind::Void {
                body_block.statements.push(SemanticStatement::Return(None));
            } else if name == "main" {
                let zero_expr = SemanticExpression {
                    kind: SemanticExpressionKind::IntegerLiteral(0),
                    sem_type: SemanticType::new(SemanticTypeKind::Integer),
                    ownership: Ownership::Trivial,
                };
                body_block.statements.push(SemanticStatement::Return(Some(zero_expr)));
            } else {
                return Err(SemanticError::InexhaustiveReturnPaths {
                    function_name: name.to_string(),
                });
            }
        }

        self.functions.insert(name.to_string(), Rc::new(SemanticFunction {
            name: name.to_string(),
            params,
            return_type: self.cur_return_type.clone(),
            body: body_block,
        }));
        Ok(())
    }

    pub(super) fn call_function(&self, name: &str, arg_exprs: &[Box<ExpressionNode>]) -> Result<SemanticExpression, SemanticError> {
        let sem_args = arg_exprs.iter()
                .map(|arg| self.eval_expr(arg))
                .collect::<Result<Vec<SemanticExpression>, SemanticError>>()?;
        if BUILTIN_FNS.contains(&name) {
            return self.call_builtin_function(name, sem_args);
        }
        if let Some(func) = self.functions.get(name) {
            let param_types: Vec<SemanticType> = func.params.iter()
                .map(|param| param.sem_type.clone())
                .collect();
            self.check_args(name, &sem_args, &param_types)?;
            Ok(SemanticExpression {
                sem_type: func.return_type.clone(),
                kind: SemanticExpressionKind::FunctionCall {
                    function: func.clone(),
                    args: sem_args,
                },
                ownership: if func.return_type.can_be_owned() {
                    Ownership::Owned
                } else {
                    Ownership::Trivial
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
                    sem_type: elem_type,
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