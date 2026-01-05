use super::*;

pub struct SemanticBlock {
    pub statements: Vec<SemanticStatement>,
    pub terminates: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LoopId(pub(super) usize);

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
            _ => false,
        }
    }
}

impl SemanticGen {
    fn find_loop_id(&self, label: &Option<String>) -> Option<LoopId> {
        match label {
            Some(label_name) => {
                self.loops.iter().rev()
                    .find(|(lbl, _)| lbl.as_ref() == Some(label_name))
                    .map(|(_, id)| *id)
            },
            None => self.loops.last().map(|(_, id)| *id)
        }
    }

    pub(super) fn next_loop_id(&mut self) -> LoopId {
        let loop_id = LoopId(self._next_loop_id);
        self._next_loop_id += 1;
        loop_id
    }

    pub(super) fn eval_block(&mut self, statements: &[StatementNode]) -> Result<SemanticBlock, SemanticError> {
        let mut sem_statements = Vec::new();
        let mut terminates = false;
        for stmt in statements {
            let sem_stmt = self.eval_stmt(stmt)?;
            terminates = sem_stmt.is_terminating();
            sem_statements.push(sem_stmt);
            if terminates {
                break;
            }
        }

        Ok(SemanticBlock {
            statements: sem_statements,
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
            let sem_block = self.eval_block(&branch.body)?;
            sem_branches.push(SemanticConditionalBranch {
                condition: sem_condition,
                body: sem_block,
            });
        }

        let else_body = match else_branch {
            Some(else_statements) => {
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

        let loop_id = self.next_loop_id();

        self.loops.push((label.clone(), loop_id));
        let sem_body = self.eval_block(body)?;
        self.loops.pop();

        Ok(SemanticStatement::ConditionalLoop {
            condition: sem_condition,
            body: sem_body,
            id: loop_id,
        })
    }

    pub(super) fn eval_return(
        &self,
        expr: &Option<Box<ExpressionNode>>,
    ) -> Result<SemanticStatement, SemanticError> {
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

        Ok(SemanticStatement::Return( sem_expr ))
    }

    pub(super) fn eval_break(&self, label: &Option<String>) -> Result<SemanticStatement, SemanticError> {
        let loop_id = self.find_loop_id(label).ok_or_else(|| {
            match label {
                Some(lbl) => SemanticError::InvalidLoopLabel { label: lbl.clone() },
                None => SemanticError::BreakOutsideLoop,
            }
        })?;
        Ok(SemanticStatement::Break(loop_id))
    }

    pub(super) fn eval_continue(&self, label: &Option<String>) -> Result<SemanticStatement, SemanticError> {
        let loop_id = self.find_loop_id(label).ok_or_else(|| {
            match label {
                Some(lbl) => SemanticError::InvalidLoopLabel { label: lbl.clone() },
                None => SemanticError::ContinueOutsideLoop,
            }
        })?;
        Ok(SemanticStatement::Continue(loop_id))
    }
}