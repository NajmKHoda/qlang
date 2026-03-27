use std::collections::HashMap;

use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Ownership {
    Owned,
    Borrowed,
    Trivial
}

pub struct SemanticStruct {
    pub name: String,
    pub id: u32,
    pub fields: Vec<(String, SemanticType)>,
}

impl SemanticStruct {
    pub fn field_index_type(&self, field_name: &str) -> Option<(usize, &SemanticType)> {
        self.fields.iter()
            .position(|(name, _)| field_name == name)
            .map(|i| (i, &self.fields[i].1))
    }
}

impl SemanticGen {
    pub fn define_struct_type(&mut self, name: &str, fields: &[TypedQNameNode]) -> Result<(), SemanticError> {
        let mut sem_fields = vec![];
        for field in fields {
            let sem_type = self.try_get_semantic_type(&field.type_node)?;
            sem_fields.push((field.name.clone(), sem_type));
        }

        let struct_id = self.struct_id_gen.next_id();
        self.structs.insert(name.to_string(), struct_id, SemanticStruct {
            name: name.to_string(),
            id: struct_id,
            fields: sem_fields
        });
        Ok(())
    }

    pub fn eval_struct(&mut self, name: Option<&str>, column_values: &[ColumnValueNode]) -> Result<SemanticExpression, SemanticError> {
        let mut fields = HashMap::new();
        for col_val in column_values {
            let sem_expr = self.eval_expr(&col_val.value)?;
            if fields.contains_key(&col_val.name) {
                return Err(SemanticError::DuplicateFieldInitialization { name: col_val.name.clone() });
            }
            fields.insert(col_val.name.clone(), sem_expr);
        }

        let mut field_types = HashMap::new();
        for (field_name, sem_expr) in &fields {
            field_types.insert(field_name.clone(), sem_expr.sem_type.clone());
        }

        let struct_type = match name {
            Some(struct_name) => {
                if let Some(named_struct) = self.structs.get_by_name(struct_name) {
                    if self.is_compatible_with_struct(named_struct.id, &field_types) {
                        SemanticType::new(SemanticTypeKind::NamedStruct(
                            named_struct.id,
                            named_struct.name.clone()
                        ))
                    } else {
                        return Err(SemanticError::IncompatibleStructInitialization {
                            struct_name: struct_name.to_string()
                        });
                    }
                } else {
                    return Err(SemanticError::UndefinedStruct { name: struct_name.to_string() });
                }
            },
            None => {
                SemanticType::new(SemanticTypeKind::AnonymousStruct(field_types))
            }
        };

        Ok(SemanticExpression {
            kind: SemanticExpressionKind::Struct(fields),
            ownership: Ownership::Trivial,
            sem_type: struct_type,
        })
    }

    pub fn eval_struct_field(&mut self, struct_expr: &ExpressionNode, field_name: &str) -> Result<SemanticExpression, SemanticError> {
        let sem_struct = self.eval_expr(struct_expr)?;
        match &sem_struct.sem_type.kind() {
            SemanticTypeKind::NamedStruct(struct_id, _) => {
                let named_struct = &self.structs[*struct_id];
                let field_info = named_struct.field_index_type(field_name);
                if let Some((position, field_type)) = field_info {
                    Ok(SemanticExpression {
                        kind: SemanticExpressionKind::StructField {
                            struct_expr: Box::new(sem_struct),
                            index: position as u32,
                        },
                        ownership: if field_type.can_be_owned() {
                            Ownership::Borrowed
                        } else {
                            Ownership::Trivial
                        },
                        sem_type: field_type.clone(),
                    })
                } else {
                    Err(SemanticError::UndefinedStructFieldAccess {
                        struct_type: sem_struct.sem_type,
                        field_name: field_name.to_string(),
                    })
                }
            }
            SemanticTypeKind::AnonymousStruct(_) => {
                Err(SemanticError::AnonymousStructFieldAccess {
                    struct_type: sem_struct.sem_type,
                    field_name: field_name.to_string(),
                })
            }
            _ => {
                Err(SemanticError::NonStructFieldAccess {
                    sem_type: sem_struct.sem_type,
                    field_name: field_name.to_string(),
                })
            }
        }
    }

    pub fn eval_array(&mut self, elements: &[Box<ExpressionNode>]) -> Result<SemanticExpression, SemanticError> {
        let elem_type = SemanticType::new(SemanticTypeKind::Any);
        let mut sem_exprs: Vec<SemanticExpression> = vec![];
        for elem in elements {
            let mut sem_expr = self.eval_expr(elem)?;
            if !self.try_unify(&sem_expr.sem_type, &elem_type) {
                return Err(SemanticError::HeterogeneousArray {
                    type_a: elem_type,
                    type_b: sem_expr.sem_type,
                });
            }
            sem_expr.sem_type = elem_type.clone();
            sem_exprs.push(sem_expr);
        }

        let array_type = SemanticType::new(SemanticTypeKind::Array(elem_type));
        Ok(SemanticExpression {
            kind: SemanticExpressionKind::Array(sem_exprs),
            sem_type: array_type,
            ownership: Ownership::Owned,
        })
    }

    pub fn eval_array_index(&mut self, array_expr: &ExpressionNode, index_expr: &ExpressionNode) -> Result<SemanticExpression, SemanticError> {
        let sem_array = self.eval_expr(array_expr)?;
        let sem_index = self.eval_expr(index_expr)?;

        if let SemanticTypeKind::Array(elem_type) = sem_array.sem_type.kind() {
            if sem_index.sem_type.kind() == SemanticTypeKind::Integer {
                Ok(SemanticExpression {
                    kind: SemanticExpressionKind::ArrayIndex {
                        array_expr: Box::new(sem_array),
                        index_expr: Box::new(sem_index),
                    },
                    sem_type: elem_type.clone(),
                    ownership: if elem_type.can_be_owned() {
                        Ownership::Borrowed
                    } else {
                        Ownership::Trivial
                    },
                })
            } else {
                Err(SemanticError::NonIntegralArrayIndex {
                    index_type: sem_index.sem_type,
                })
            }
        } else {
            Err(SemanticError::NonArrayIndex {
                sem_type: sem_array.sem_type,
            })
        }
    }
}