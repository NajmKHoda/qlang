use std::rc::Rc;

use super::*;

#[derive(Clone)]
pub(super) struct SemanticVariable {
    pub(super) sem_type: SemanticType
}

impl SemanticGen {
    pub fn define_variable(&mut self, name: &str, type_node: &TypeNode, init_expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
        let sem_type = self.try_get_semantic_type(type_node)?;
        let sem_init_expr = self.eval_expr(init_expr)?;
        let compatible = SemanticType::try_unify(&sem_type, &sem_init_expr.sem_type);
        if !compatible {
            return Err(SemanticError::IncompatibleAssignment {
                var_name: name.to_string(),
                var_type: sem_type,
                expr_type: sem_init_expr.sem_type
            });
        }
        if !sem_type.is_concrete() {
            return Err(SemanticError::AmbiguousVariableType {
                var_name: name.to_string(),
                var_type: sem_type
            });
        }

        let current_scope = self.variables.last_mut().unwrap();
        if current_scope.contains_key(name) {
            return Err(SemanticError::DuplicateVariableDefinition { name: name.to_string() })
        }

        let variable = Rc::new(SemanticVariable { sem_type: sem_type.clone() });
        current_scope.insert(name.to_string(), variable.clone());
        Ok(SemanticStatement::VariableDeclaration { variable, init_expr: sem_init_expr })
    }

    pub fn assign_variable(&self, name: &str, expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
        let variable = self.get_variable(name)?;
        let sem_expr = self.eval_expr(expr)?;
        let compatible = sem_expr.sem_type.try_downcast(&variable.sem_type);
        if !compatible {
            return Err(SemanticError::IncompatibleAssignment {
                var_name: name.to_string(),
                var_type: variable.sem_type.clone(),
                expr_type: sem_expr.sem_type
            });
        }
        Ok(SemanticStatement::VariableAssignment { variable, expr: sem_expr })
    }

    pub fn get_variable(&self, name: &str) -> Result<Rc<SemanticVariable>, SemanticError> {
        for scope in self.variables.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(var.clone());
            }
        }
        Err(SemanticError::UndefinedVariable { name: name.to_string() })
    }
}