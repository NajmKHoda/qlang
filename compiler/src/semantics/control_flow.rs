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
        statements: &[StatementNode],
        scope_type: SemanticScopeType
    ) -> Result<SemanticBlock, SemanticError> {
        self.enter_scope(scope_type);

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
            let sem_block = self.eval_block(&branch.body, SemanticScopeType::Block)?;
            sem_branches.push(SemanticConditionalBranch {
                condition: sem_condition,
                body: sem_block,
            });
        }

        let else_body = match else_branch {
            Some(else_statements) => {
                let sem_else_block = self.eval_block(else_statements, SemanticScopeType::Block)?;
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
        let sem_body = self.eval_block(body, SemanticScopeType::Loop(loop_id))?;
        self.loops.pop();

        Ok(SemanticStatement::ConditionalLoop {
            condition: sem_condition,
            body: sem_body,
            id: loop_id,
        })
    }

    pub(super) fn eval_return(
        &self,
        expr: Option<&ExpressionNode>,
    ) -> Result<Vec<SemanticStatement>, SemanticError> {
        let sem_expr = match expr {
            Some(expr_node) => {
                let sem_expr = self.eval_expr(expr_node)?;
                if self.cur_return_type != sem_expr.sem_type {
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

        let mut stmts = self.drop_to_scope(SemanticScopeType::Function)?;
        let return_stmt = SemanticStatement::Return(sem_expr);
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

        let loop_scope = SemanticScopeType::Loop(loop_id);
        let mut stmts = self.drop_to_scope(loop_scope)?;
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

        let loop_scope = SemanticScopeType::Loop(loop_id);
        let mut stmts = self.drop_to_scope(loop_scope)?;
        let continue_stmt = SemanticStatement::Continue(loop_id);
        stmts.push(continue_stmt);

        Ok(stmts)
    }
}