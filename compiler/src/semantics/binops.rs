use super::*;

impl SemanticGen {
    pub(super) fn eval_add(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;
        
        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
            (SemanticTypeKind::Float, SemanticTypeKind::Float) => {},
            (SemanticTypeKind::String, SemanticTypeKind::String) => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "addition".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: sem_left.sem_type.clone(),
            ownership: if sem_left.sem_type.can_be_owned() {
                Ownership::Owned
            } else {
                Ownership::Trivial
            },
            kind: SemanticExpressionKind::Add {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_subtract(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
            (SemanticTypeKind::Float, SemanticTypeKind::Float) => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "subtraction".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: sem_left.sem_type.clone(),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::Subtract {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_multiply(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
            (SemanticTypeKind::Float, SemanticTypeKind::Float) => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "multiplication".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: sem_left.sem_type.clone(),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::Multiply {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_divide(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
            (SemanticTypeKind::Float, SemanticTypeKind::Float) => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "division".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: sem_left.sem_type.clone(),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::Divide {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_modulus(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "modulus".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: sem_left.sem_type.clone(),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::Modulus {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_logical_and(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool) => {}
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "logical and".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Bool),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::LogicalAnd {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_logical_or(&mut self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool) => {}
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "logical or".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Bool),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::LogicalOr {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_logical_not(&mut self, value: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_value = self.eval_expr(value)?;

        match sem_value.sem_type.kind() {
            SemanticTypeKind::Bool => {}
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "logical not".to_string(),
                    left_type: sem_value.sem_type.clone(),
                    right_type: SemanticType::new(SemanticTypeKind::Bool),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Bool),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::LogicalNot {
                value: Box::new(sem_value),
            },
        })
    }

    pub(super) fn eval_compare(
        &mut self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        op: ComparisonType
    ) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) |
            (SemanticTypeKind::Float, SemanticTypeKind::Float) |
            (SemanticTypeKind::String, SemanticTypeKind::String) => {},
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool)
                if op == ComparisonType::Equal || op == ComparisonType::NotEqual => {},
            _ => {
                return Err(SemanticError::IncompatibleOperands {
                    operation: "comparison".to_string(),
                    left_type: sem_left.sem_type.clone(),
                    right_type: sem_right.sem_type.clone(),
                });
            }
        }

        Ok(SemanticExpression {
            sem_type: SemanticType::new(SemanticTypeKind::Bool),
            ownership: Ownership::Trivial,
            kind: SemanticExpressionKind::Compare {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
                op,
            },
        })
    }
}