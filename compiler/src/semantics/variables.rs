use super::*;

#[derive(Clone)]
pub struct SemanticVariable {
    pub name: String,
    pub id: u32,
    pub sem_type: SemanticType,
}

impl SemanticGen {
    pub fn define_variable(&mut self, name: &str, type_node: &Option<TypeNode>, init_expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
        let sem_init_expr = self.eval_expr(init_expr)?;

        let sem_type = match type_node {
            Some(t) => {
                let sem_type = self.try_get_semantic_type(t)?;
                let compatible = self.try_unify(&sem_type, &sem_init_expr.sem_type);
                if !compatible {
                    return Err(SemanticError::IncompatibleAssignment {
                        var_name: name.to_string(),
                        var_type: sem_type,
                        expr_type: sem_init_expr.sem_type
                    });
                }
                sem_type
            },
            None => sem_init_expr.sem_type.clone(),
        };

        if !sem_type.is_concrete() {
            return Err(SemanticError::AmbiguousVariableType {
                var_name: name.to_string(),
                var_type: sem_type
            });
        }

        let current_scope = self.variable_scopes.last_mut().unwrap();
        if current_scope.contains_key(name) {
            return Err(SemanticError::DuplicateVariableDefinition {
                name: name.to_string()
            })
        }

        let variable_id = self.variable_id_gen.next_id();
        current_scope.insert(name.to_string(), variable_id);
        self.variables.insert(variable_id, SemanticVariable {
            name: name.to_string(),
            id: variable_id,
            sem_type,
        });
        let declaration_node = SemanticStatement::VariableDeclaration {
            variable_id,
            init_expr: sem_init_expr,
        };

        Ok(declaration_node)
    }

    pub fn assign_variable(&self, name: &str, expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
        let variable = self.get_variable(name)?;
        let sem_expr = self.eval_expr(expr)?;
        let compatible = self.try_downcast(&variable.sem_type, &sem_expr.sem_type);
        if !compatible {
            return Err(SemanticError::IncompatibleAssignment {
                var_name: name.to_string(),
                var_type: variable.sem_type.clone(),
                expr_type: sem_expr.sem_type
            });
        }
        Ok(SemanticStatement::VariableAssignment {
            variable_id: variable.id,
            expr: sem_expr
        })
    }

    pub fn get_variable(&self, name: &str) -> Result<&SemanticVariable, SemanticError> {
        for scope in self.variable_scopes.iter().rev() {
            if let Some(var_id) = scope.get(name) {
                return Ok(&self.variables[var_id])
            }
        }
        Err(SemanticError::UndefinedVariable { name: name.to_string() })
    }
}

