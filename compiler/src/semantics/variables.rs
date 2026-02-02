use super::*;

#[derive(Clone)]
pub struct SemanticVariable {
    pub name: String,
    pub id: u32,
    pub sem_type: SemanticType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticScopeType {
    Function,
    Loop(u32),
    Block,
}

pub(super) struct SemanticScope {
    pub(super) variables: HashMap<String, u32>,
    pub(super) scope_type: SemanticScopeType,
}

impl SemanticGen {
    pub(super) fn define_variable(&mut self, name: &str, type_node: &Option<TypeNode>, init_expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
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

        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.variables.contains_key(name) {
            return Err(SemanticError::DuplicateVariableDefinition {
                name: name.to_string()
            })
        }

        let variable_id = self.variable_id_gen.next_id();
        current_scope.variables.insert(name.to_string(), variable_id);
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

    pub(super) fn assign_variable(&self, name: &str, expr: &ExpressionNode) -> Result<SemanticStatement, SemanticError> {
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

    pub(super) fn get_variable(&self, name: &str) -> Result<&SemanticVariable, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_id) = scope.variables.get(name) {
                return Ok(&self.variables[var_id])
            }
        }
        Err(SemanticError::UndefinedVariable { name: name.to_string() })
    }

    pub(super) fn enter_scope(&mut self, scope_type: SemanticScopeType) {
        self.scopes.push(SemanticScope {
            variables: HashMap::new(),
            scope_type,
        });
    }

    pub(super) fn exit_scope(&mut self, drop_vars: bool) -> Vec<SemanticStatement> {
        let cur_scope = self.scopes.last().unwrap();
        let drop_stmts = if drop_vars {
            cur_scope.variables.values().map(|var_id| {
                SemanticStatement::DropVariable(*var_id)
            }).collect::<Vec<SemanticStatement>>()
        } else {
            vec![]
        };

        self.scopes.pop();
        drop_stmts
    }
}

