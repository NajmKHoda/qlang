use super::*;

impl SemanticGen {
    pub(super) fn eval_add(&self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;
        
        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
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
            kind: SemanticExpressionKind::Add {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_subtract(&self, left: &ExpressionNode, right: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => {},
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
            kind: SemanticExpressionKind::Subtract {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
            },
        })
    }

    pub(super) fn eval_compare(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        op: ComparisonType
    ) -> Result<SemanticExpression, SemanticError> {
        let sem_left = self.eval_expr(left)?;
        let sem_right = self.eval_expr(right)?;

        match (&sem_left.sem_type.kind(), &sem_right.sem_type.kind()) {
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) |
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
            kind: SemanticExpressionKind::Compare {
                left: Box::new(sem_left),
                right: Box::new(sem_right),
                op,
            },
        })
    }
}