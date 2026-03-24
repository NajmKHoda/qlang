use super::*;

pub struct SemanticBlock {
    pub statements: Vec<SemanticStatement>,
    pub terminates: bool,
}

impl SemanticStatement {
    fn is_terminating(&self) -> bool {
        match self {
            SemanticStatement::Conditional { branches, else_branch } => {
                let all_branches_terminate = branches.iter().all(|branch| {
                    branch.body.terminates
                });
                let else_terminates = match else_branch {
                    Some(else_body) => else_body.terminates,
                    None => false,
                };
                all_branches_terminate && else_terminates
            }
            SemanticStatement::Return(_) => true,
            SemanticStatement::Break(_) => true,
            SemanticStatement::Continue(_) => true,
            _ => false,
        }
    }
}

impl SemanticGen {
    fn find_loop_id(&self, label: &Option<String>) -> Option<u32> {
        match label {
            Some(label_name) => {
                self.loops.iter().rev()
                    .find(|(lbl, _)| lbl.as_ref() == Some(label_name))
                    .map(|(_, id)| *id)
            },
            None => self.loops.last().map(|(_, id)| *id)
        }
    }

    pub(super) fn eval_block(
        &mut self,
        statements: &[StatementNode]
    ) -> Result<SemanticBlock, SemanticError> {
        // Evaluate statements in this block
        let mut sem_stmts: Vec<SemanticStatement> = vec![];
        let mut terminates = false;
        for stmt in statements {
            let mut cur_stmts = self.eval_stmt(stmt)?;
            terminates = match cur_stmts.last() {
                Some(last_stmt) => last_stmt.is_terminating(),
                None => false,
            };
            sem_stmts.append(&mut cur_stmts);
            if terminates {
                break;
            }
        }

        // Drop variables in this scope
        self.exit_scope(!terminates)
            .into_iter()
            .for_each(|drop_stmt| sem_stmts.push(drop_stmt));

        Ok(SemanticBlock {
            statements: sem_stmts,
            terminates,
        })
    }

    pub(super) fn eval_conditional(
        &mut self,
        branches: &[ConditionalBranchNode],
        else_branch: &Option<Vec<StatementNode>>
    ) -> Result<SemanticStatement, SemanticError> {
        let mut sem_branches = Vec::new();
        for branch in branches {
            let sem_condition = self.eval_expr(&branch.condition)?;
            if sem_condition.sem_type != SemanticTypeKind::Bool {
                return Err(SemanticError::NonBoolCondition {
                    found_type: sem_condition.sem_type.clone(),
                });
            }

            self.enter_scope(SemanticScopeType::Block);
            let sem_block = self.eval_block(&branch.body)?;
            sem_branches.push(SemanticConditionalBranch {
                condition: sem_condition,
                body: sem_block,
            });
        }

        let else_body = match else_branch {
            Some(else_statements) => {
                self.enter_scope(SemanticScopeType::Block);
                let sem_else_block = self.eval_block(else_statements)?;
                Some(sem_else_block)
            },
            None => None,
        };

        Ok(SemanticStatement::Conditional {
            branches: sem_branches,
            else_branch: else_body,
        })
    }

    pub(super) fn eval_conditional_loop(
        &mut self,
        condition: &ExpressionNode,
        body: &[StatementNode],
        label: &Option<String>,
    ) -> Result<SemanticStatement, SemanticError> {
        let sem_condition = self.eval_expr(condition)?;
        if sem_condition.sem_type != SemanticTypeKind::Bool {
            return Err(SemanticError::NonBoolCondition {
                found_type: sem_condition.sem_type.clone(),
            });
        }

        let loop_id = self.loop_id_gen.next_id();

        self.loops.push((label.clone(), loop_id));
        self.enter_scope(SemanticScopeType::Loop(loop_id));
        let sem_body = self.eval_block(body)?;
        self.loops.pop();

        Ok(SemanticStatement::ConditionalLoop {
            condition: sem_condition,
            body: sem_body,
            id: loop_id,
        })
    }

    pub(super) fn eval_for_loop(
        &mut self,
        loop_var_name: &str,
        iterable_expr: &ExpressionNode,
        body: &[StatementNode],
        label: &Option<String>,
    ) -> Result<Vec<SemanticStatement>, SemanticError> {
        let mut stmts: Vec<SemanticStatement> = vec![];

        let sem_iterable = self.eval_expr(iterable_expr)?;
        let (iterator, loop_var_type) = match sem_iterable.sem_type.kind() {
            SemanticTypeKind::Iterator(elem_type) => (sem_iterable, elem_type.clone()),
            SemanticTypeKind::Array(elem_type) => {
                (SemanticExpression {
                    kind: SemanticExpressionKind::BuiltinMethodCall {
                        receiver: Box::new(sem_iterable),
                        method: BuiltinMethod::ArrayIter,
                        args: Vec::new()
                    },
                    sem_type: SemanticType::new(SemanticTypeKind::Iterator(elem_type.clone())),
                    ownership: Ownership::Owned,
                }, elem_type.clone())
            }
            _ => return Err(SemanticError::NonIterableExpression {
                found_type: sem_iterable.sem_type.clone(),
            }),
        };

        // Create loop variable
        let iterator_type = SemanticType::new(SemanticTypeKind::Iterator(loop_var_type.clone()));
        let iterator_var_id = self.variable_id_gen.next_id();
        self.variables.insert(iterator_var_id, SemanticVariable {
            name: format!("__ql__iterator_{}", iterator_var_id),
            id: iterator_var_id,
            sem_type: iterator_type.clone(),
        });
        stmts.push(SemanticStatement::VariableDeclaration {
            variable_id: iterator_var_id,
            init_expr: iterator,
        });

        let loop_id = self.loop_id_gen.next_id();
        self.loops.push((label.clone(), loop_id));
        self.enter_scope(SemanticScopeType::Loop(loop_id));

        let loop_var_id = self.variable_id_gen.next_id();
        self.scopes.last_mut().unwrap().variables.insert(loop_var_name.to_string(), loop_var_id);
        self.variables.insert(loop_var_id, SemanticVariable {
            name: loop_var_name.to_string(),
            id: loop_var_id,
            sem_type: loop_var_type.clone(),
        });

        let mut sem_body = self.eval_block(body)?;
        sem_body.statements.insert(0, SemanticStatement::VariableDeclaration {
            variable_id: loop_var_id,
            init_expr: SemanticExpression {
                kind: SemanticExpressionKind::BuiltinMethodCall {
                    receiver: Box::new(SemanticExpression {
                        kind: SemanticExpressionKind::Variable(iterator_var_id),
                        sem_type: iterator_type.clone(),
                        ownership: Ownership::Borrowed,
                    }),
                    method: BuiltinMethod::IteratorNext,
                    args: Vec::new()
                },
                sem_type: loop_var_type.clone(),
                ownership: Ownership::Borrowed,
            },
        });
        self.loops.pop();

        let condition = SemanticExpression {
            kind: SemanticExpressionKind::BuiltinMethodCall {
                receiver: Box::new(SemanticExpression {
                    kind: SemanticExpressionKind::Variable(iterator_var_id),
                    sem_type: iterator_type.clone(),
                    ownership: Ownership::Borrowed,
                }),
                method: BuiltinMethod::IteratorHasNext,
                args: Vec::new()
            },
            sem_type: SemanticType::new(SemanticTypeKind::Bool),
            ownership: Ownership::Trivial,
        };
        
        stmts.push(SemanticStatement::ConditionalLoop {
            condition,
            body: sem_body,
            id: loop_id,
        });
        stmts.push(SemanticStatement::DropVariable(iterator_var_id));

        Ok(stmts)
    }

    pub(super) fn eval_return(
        &mut self,
        expr: Option<&ExpressionNode>,
    ) -> Result<Vec<SemanticStatement>, SemanticError> {
        let sem_expr_op = match expr {
            Some(expr_node) => {
                let sem_expr = self.eval_expr(expr_node)?;
                if !self.try_unify(&self.cur_return_type, &sem_expr.sem_type) {
                    return Err(SemanticError::MistypedReturnValue {
                        expected: self.cur_return_type.clone(),
                        found: sem_expr.sem_type,
                    });
                }
                Some(sem_expr)
            }
            None => {
                if self.cur_return_type != SemanticTypeKind::Void {
                    return Err(SemanticError::MistypedReturnValue {
                        expected: self.cur_return_type.clone(),
                        found: SemanticType::new(SemanticTypeKind::Void),
                    });
                }
                None
            }
        };

        let mut stmts: Vec<SemanticStatement> = vec![];
        let return_var_id = if let Some(sem_expr) = sem_expr_op {
            if sem_expr.sem_type == SemanticTypeKind::Void {
                stmts.push(SemanticStatement::LoneExpression(sem_expr));
                None
            } else {
                let return_var_id = self.variable_id_gen.next_id();
                self.variables.insert(return_var_id, SemanticVariable {
                    name: format!("__ql__ret_{}", return_var_id),
                    id: return_var_id,
                    sem_type: sem_expr.sem_type.clone(),
                });
                stmts.push(SemanticStatement::VariableDeclaration {
                    variable_id: return_var_id,
                    init_expr: sem_expr,
                });
                Some(return_var_id)
            }
        } else {
            None
        };

        // Drop variables up to (but not including) functional scope
        for scope in self.scopes.iter().rev() {
            match scope.scope_type {
                SemanticScopeType::Function
                | SemanticScopeType::Closure(_) => break,
                _ => {},
            }
            for var_id in scope.variables.values() {
                let drop_stmt = SemanticStatement::DropVariable(*var_id);
                stmts.push(drop_stmt);
            }
        }
        let return_stmt = SemanticStatement::Return(return_var_id);
        stmts.push(return_stmt);

        Ok(stmts)
    }

    pub(super) fn eval_break(&self, label: &Option<String>) -> Result<Vec<SemanticStatement>, SemanticError> {
        let loop_id = self.find_loop_id(label).ok_or_else(|| {
            match label {
                Some(lbl) => SemanticError::InvalidLoopLabel { label: lbl.clone() },
                None => SemanticError::BreakOutsideLoop,
            }
        })?;

        // Drop variables in the current loop scope
        let mut stmts: Vec<SemanticStatement> = vec![];
        for scope in self.scopes.iter().rev() {
            for var_id in scope.variables.values() {
                let drop_stmt = SemanticStatement::DropVariable(*var_id);
                stmts.push(drop_stmt);
            }
            if scope.scope_type == SemanticScopeType::Loop(loop_id) {
                break;
            }
        }

        // Emit the break statement
        let break_stmt = SemanticStatement::Break(loop_id);
        stmts.push(break_stmt);

        Ok(stmts)
    }

    pub(super) fn eval_continue(&self, label: &Option<String>) -> Result<Vec<SemanticStatement>, SemanticError> {
        let loop_id = self.find_loop_id(label).ok_or_else(|| {
            match label {
                Some(lbl) => SemanticError::InvalidLoopLabel { label: lbl.clone() },
                None => SemanticError::ContinueOutsideLoop,
            }
        })?;

        // Drop variables in the current loop scope
        let mut stmts: Vec<SemanticStatement> = vec![];
        for scope in self.scopes.iter().rev() {
            for var_id in scope.variables.values() {
                let drop_stmt = SemanticStatement::DropVariable(*var_id);
                stmts.push(drop_stmt);
            } 
            if scope.scope_type == SemanticScopeType::Loop(loop_id) {
                break;
            }
        }

        // Emit the continue statement
        let continue_stmt = SemanticStatement::Continue(loop_id);
        stmts.push(continue_stmt);

        Ok(stmts)
    }
}