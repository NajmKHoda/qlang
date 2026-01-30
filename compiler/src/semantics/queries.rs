use std::{collections::HashMap};

use super::*;

pub struct SemanticDatasource {
    pub is_readonly: bool,
    pub id: u32,
}

pub struct SemanticTable {
    pub name: String,
    pub id: u32,
    pub is_readonly: bool,
    pub struct_id: u32,
    pub datasource_id: u32,
}

impl SemanticGen {
    fn eval_where_clause(&self, where_node: &WhereNode, table: &SemanticTable) -> Result<WhereClause, SemanticError> {
        let sem_expr = self.eval_expr(&where_node.value)?;
        let column_type = self.structs[table.struct_id].fields.get(&where_node.column_name);
        match column_type {
            Some(col_type) => {
                let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                if !compatible {
                    return Err(SemanticError::IncompatibleColumnValue {
                        table_name: table.name.clone(),
                        column_name: where_node.column_name.clone(),
                        expected: col_type.clone(),
                        found: sem_expr.sem_type.clone(),
                    });
                }
            },
            None => {
                return Err(SemanticError::UndefinedColumn {
                    table_name: table.name.clone(),
                    column_name: where_node.column_name.clone(),
                });
            }
        }
        Ok(WhereClause {
            column_name: where_node.column_name.clone(),
            value: Box::new(sem_expr),
        })
    }

    pub(super) fn declare_datasource(&mut self, name: &str, is_readonly: bool) -> Result<(), SemanticError> {
        if self.datasources.contains_name(name) {
            return Err(SemanticError::DuplicateDatasourceDeclaration {
                name: name.to_string()
            });
        }
        let datasource_id = self.datasource_id_gen.next_id();
        self.datasources.insert(name.to_string(), datasource_id, SemanticDatasource {
            is_readonly,
            id: datasource_id,
        });
        Ok(())
    }

    pub(super) fn define_table(
        &mut self,
        name: &str,
        fields: &[TypedQNameNode],
        is_readonly: bool,
        datasource_name: &str
    ) -> Result<(), SemanticError> {
        if self.tables.contains_name(name) {
            return Err(SemanticError::DuplicateTableDefinition { name: name.to_string() });
        }

        let datasource = self.datasources.get_by_name(datasource_name)
            .ok_or_else(|| SemanticError::UndefinedDatasource {
                name: datasource_name.to_string()
            })?;
        if !is_readonly && datasource.is_readonly {
            return Err(SemanticError::DatasourceReadonly {
                datasource_name: datasource_name.to_string(),
                table_name: name.to_string(),
            });
        }

        let mut struct_fields = HashMap::new();
        for field in fields {
            let is_primitive = match field.type_node {
                TypeNode::Integer | TypeNode::Bool | TypeNode::String => true,
                _ => false,
            };
            if !is_primitive {
                return Err(SemanticError::NonPrimitiveColumnType { 
                    table_name: name.to_string(),
                    column_name: field.name.clone()
                });
            }
            struct_fields.insert(
                field.name.clone(),
                self.try_get_semantic_type(&field.type_node)?,
            );
        }

        let struct_id = self.struct_id_gen.next_id();
        self.structs.insert(name.to_string(), struct_id, SemanticStruct {
            name: name.to_string(),
            id: struct_id,
            fields: struct_fields,
            field_order: fields.iter().map(|f| f.name.clone()).collect(),
        });

        let table_id = self.table_id_gen.next_id();
        self.tables.insert(name.to_string(), table_id, SemanticTable {
            name: name.to_string(),
            id: table_id,
            datasource_id: datasource.id,
            struct_id,
            is_readonly,
        });

        Ok(())
    }

    pub(super) fn eval_select_query(&self, query: &SelectQueryNode) -> Result<SemanticExpression, SemanticError> {
        if let Some(table) = self.tables.get_by_name(&query.table_name) {
            let where_clause = query.where_clause.as_ref().map(|where_node| {
                self.eval_where_clause(where_node, table)
            }).transpose()?;
            Ok(SemanticExpression {
                kind: SemanticExpressionKind::ImmediateQuery(
                    SemanticQuery::Select {
                        table_id: table.id,
                        where_clause,
                    }
                ),
                sem_type: SemanticType::new(SemanticTypeKind::Array(
                    SemanticType::new(SemanticTypeKind::NamedStruct(
                        table.struct_id,
                        self.structs[table.struct_id].name.clone()
                    ))
                )),
                ownership: Ownership::Trivial,
            })
        } else {
            Err(SemanticError::UndefinedTable { name: query.table_name.clone() })
        }
    }

    pub(super) fn eval_insert_query(&self, query: &InsertQueryNode) -> Result<SemanticExpression, SemanticError> {
        if let Some(table) = self.tables.get_by_name(&query.table_name) {
            if table.is_readonly {
                return Err(SemanticError::ReadonlyTableMutation {
                    table_name: table.name.clone(),
                    operation: "INSERT",
                });
            }

            let sem_value = self.eval_expr(&query.data_expr)?;
            let expected_type = SemanticType::new(SemanticTypeKind::NamedStruct(
                table.struct_id,
                self.structs[table.struct_id].name.clone()
            ));
            let compatible = self.try_downcast(&expected_type, &sem_value.sem_type);
            if !compatible {
                return Err(SemanticError::IncompatibleInsertData {
                    table_name: table.name.clone(),
                    found_type: sem_value.sem_type.clone()
                });
            }

            Ok(SemanticExpression {
                kind: SemanticExpressionKind::ImmediateQuery(
                    SemanticQuery::Insert {
                        table_id: table.id,
                        value: Box::new(sem_value),
                    }
                ),
                sem_type: SemanticType::new(SemanticTypeKind::Void),
                ownership: Ownership::Trivial,
            })
        } else {
            Err(SemanticError::UndefinedTable { name: query.table_name.clone() })
        }
    }

    pub(super) fn eval_update_query(&self, query: &UpdateQueryNode) -> Result<SemanticExpression, SemanticError> {
        if let Some(table) = self.tables.get_by_name(&query.table_name) {
            if table.is_readonly {
                return Err(SemanticError::ReadonlyTableMutation {
                    table_name: table.name.clone(),
                    operation: "UPDATE",
                });
            }

            let assignments = query.assignments.iter().map(|UpdateAssignmentNode { column_name, value_expr }| {
                let sem_expr = self.eval_expr(value_expr)?;
                let table_struct = &self.structs[table.struct_id];
                let column_type = table_struct.fields.get(column_name);
                match column_type {
                    Some(col_type) => {
                        let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                        if !compatible {
                            return Err(SemanticError::IncompatibleColumnValue {
                                table_name: table.name.clone(),
                                column_name: column_name.clone(),
                                expected: col_type.clone(),
                                found: sem_expr.sem_type.clone(),
                            });
                        }
                    },
                    None => {
                        return Err(SemanticError::UndefinedColumn {
                            table_name: table.name.clone(),
                            column_name: column_name.clone(),
                        });
                    }
                }
                Ok((column_name.clone(), sem_expr))
            }).collect::<Result<HashMap<String, SemanticExpression>, SemanticError>>()?;

            let where_clause = query.where_clause.as_ref().map(|where_node| {
                self.eval_where_clause(where_node, table)
            }).transpose()?;

            Ok(SemanticExpression {
                kind: SemanticExpressionKind::ImmediateQuery(
                    SemanticQuery::Update {
                        table_id: table.id,
                        assignments,
                        where_clause,
                    }
                ),
                sem_type: SemanticType::new(SemanticTypeKind::Void),
                ownership: Ownership::Trivial,
            })
        } else {
            Err(SemanticError::UndefinedTable { name: query.table_name.clone() })
        }
    }

    pub(super) fn eval_delete_query(&self, query: &DeleteQueryNode) -> Result<SemanticExpression, SemanticError> {
        if let Some(table) = self.tables.get_by_name(&query.table_name) {
            if table.is_readonly {
                return Err(SemanticError::ReadonlyTableMutation {
                    table_name: table.name.clone(),
                    operation: "DELETE",
                });
            }

            let where_clause = query.where_clause.as_ref().map(|where_node| {
                self.eval_where_clause(where_node, table)
            }).transpose()?;
            Ok(SemanticExpression {
                kind: SemanticExpressionKind::ImmediateQuery(
                    SemanticQuery::Delete {
                        table_id: table.id,
                        where_clause,
                    }
                ),
                sem_type: SemanticType::new(SemanticTypeKind::Void),
                ownership: Ownership::Trivial,
            })
        } else {
            Err(SemanticError::UndefinedTable { name: query.table_name.clone() })
        }
    }
}