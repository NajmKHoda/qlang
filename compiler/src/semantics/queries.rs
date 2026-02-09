use std::{collections::HashMap};

use super::*;

pub struct SemanticDatasource {
    pub name: String,
    pub id: u32,
    pub is_readonly: bool,
}

pub struct SemanticTable {
    pub name: String,
    pub id: u32,
    pub is_readonly: bool,
    pub struct_id: u32,
    pub datasource_id: u32,
}

impl SemanticGen {
    fn eval_where_clause(&self, table: &SemanticTable, column_name: &str, sem_expr: SemanticExpression) -> Result<WhereClause, SemanticError> {
        let table_struct = &self.structs[table.struct_id];
        let column_type = table_struct.fields.get(column_name);
        match column_type {
            Some(col_type) => {
                let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                if !compatible {
                    return Err(SemanticError::IncompatibleColumnValue {
                        table_name: table.name.clone(),
                        column_name: column_name.to_string(),
                        expected: col_type.clone(),
                        found: sem_expr.sem_type.clone(),
                    });
                }
            },
            None => {
                return Err(SemanticError::UndefinedColumn {
                    table_name: table.name.clone(),
                    column_name: column_name.to_string(),
                });
            }
        }

        let column_index = table_struct.field_order.iter()
            .position(|name| name == column_name)
            .unwrap() as u32;
        Ok(WhereClause {
            column_index,
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
            name: name.to_string(),
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

    fn eval_select_query(&mut self, query: &SelectQueryNode) -> Result<SemanticQuery, SemanticError> {
        let where_expr = query.where_clause.as_ref().map(|where_node| {
            let sem_expr = self.eval_expr(&where_node.value)?;
            Ok((where_node.column_name.clone(), sem_expr))
         }).transpose()?;

        let table = self.tables.get_by_name(&query.table_name)
            .ok_or_else(|| SemanticError::UndefinedTable { name: query.table_name.clone() })?;

        let where_clause = match where_expr {
            Some((column_name, sem_expr)) => Some(self.eval_where_clause(table, &column_name, sem_expr)?),
            None => None,
        };

        Ok(SemanticQuery::Select {
            table_id: table.id,
            where_clause,
        })
    }

    fn eval_insert_query(&mut self, query: &InsertQueryNode) -> Result<SemanticQuery, SemanticError> {
        let sem_value = self.eval_expr(&query.data_expr)?;

        let table = self.tables.get_by_name(&query.table_name)
            .ok_or_else(|| SemanticError::UndefinedTable { name: query.table_name.clone() })?;
        if table.is_readonly {
            return Err(SemanticError::ReadonlyTableMutation {
                table_name: table.name.clone(),
                operation: "INSERT",
            });
        }

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

        Ok(SemanticQuery::Insert {
            table_id: table.id,
            value: Box::new(sem_value),
        })
    }

    fn eval_update_query(&mut self, query: &UpdateQueryNode) -> Result<SemanticQuery, SemanticError> {
        let assignments: Vec<(&str, SemanticExpression)> = query.assignments
            .iter()
            .map(|assignment| {
                let sem_expr = self.eval_expr(&assignment.value_expr)?;
                Ok((assignment.column_name.as_str(), sem_expr))
            })
            .collect::<Result<_, SemanticError>>()?;

        let where_expr = query.where_clause.as_ref().map(|where_node| {
            let sem_expr = self.eval_expr(&where_node.value)?;
            Ok((where_node.column_name.clone(), sem_expr))
        }).transpose()?;

        let Some(table) = self.tables.get_by_name(&query.table_name) else {
            return Err(SemanticError::UndefinedTable { name: query.table_name.clone() });
        };
        if table.is_readonly {
            return Err(SemanticError::ReadonlyTableMutation {
                table_name: table.name.clone(),
                operation: "UPDATE",
            });
        }

        let where_clause = match where_expr {
            Some((column_name, sem_expr)) => Some(self.eval_where_clause(table, &column_name, sem_expr)?),
            None => None,
        };

        let table_struct = &self.structs[table.struct_id];
        let sem_assignments = assignments.into_iter().map(|(col_name, sem_expr)| {
            let column_type = table_struct.fields.get(col_name);
            match column_type {
                Some(col_type) => {
                    let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                    if !compatible {
                        return Err(SemanticError::IncompatibleColumnValue {
                            table_name: table.name.clone(),
                            column_name: col_name.to_string(),
                            expected: col_type.clone(),
                            found: sem_expr.sem_type.clone(),
                        });
                    }
                },
                None => {
                    return Err(SemanticError::UndefinedColumn {
                        table_name: table.name.clone(),
                        column_name: col_name.to_string(),
                    });
                }
            }

            let column_index = table_struct.field_order.iter()
                .position(|name| name == col_name)
                .unwrap() as u32;
            Ok(UpdateAssignment {
                column_index,
                value: sem_expr,
            })
        }).collect::<Result<Vec<_>, SemanticError>>()?;

        Ok(SemanticQuery::Update {
            table_id: table.id,
            assignments: sem_assignments,
            where_clause,
        })
    }

    fn eval_delete_query(&mut self, query: &DeleteQueryNode) -> Result<SemanticQuery, SemanticError> {
        let where_expr = query.where_clause.as_ref().map(|where_node| {
            let sem_expr = self.eval_expr(&where_node.value)?;
            Ok((where_node.column_name.clone(), sem_expr))
         }).transpose()?;

        let table = self.tables.get_by_name(&query.table_name)
            .ok_or_else(|| SemanticError::UndefinedTable { name: query.table_name.clone() })?;
        if table.is_readonly {
            return Err(SemanticError::ReadonlyTableMutation {
                table_name: table.name.clone(),
                operation: "DELETE",
            });
        }

        let where_clause = match where_expr {
            Some((column_name, sem_expr)) => Some(self.eval_where_clause(table, &column_name, sem_expr)?),
            None => None,
        };

        Ok(SemanticQuery::Delete {
            table_id: table.id,
            where_clause,
        })
    }

    fn eval_query(&mut self, query: &QueryNode) -> Result<SemanticQuery, SemanticError> {
        match query {
            QueryNode::Select(select) => self.eval_select_query(select),
            QueryNode::Insert(insert) => self.eval_insert_query(insert),
            QueryNode::Update(update) => self.eval_update_query(update),
            QueryNode::Delete(delete) => self.eval_delete_query(delete),
        }
    }

    fn return_type_of_query(&self, query: &SemanticQuery) -> SemanticType {
        match query {
            SemanticQuery::Select { table_id, .. } => {
                let table = &self.tables[*table_id];
                let table_struct = &self.structs[table.struct_id];
                let struct_type = SemanticType::new(SemanticTypeKind::NamedStruct(
                    table.struct_id,
                    table_struct.name.clone(),
                ));
                SemanticType::new(SemanticTypeKind::Array(struct_type))
            },
            _ => SemanticType::new(SemanticTypeKind::Void),
        }
    }

    pub(super) fn eval_immediate_query(&mut self, query: &QueryNode) -> Result<SemanticExpression, SemanticError> {
        let sem_query = self.eval_query(query)?;

        Ok(SemanticExpression {
            sem_type: self.return_type_of_query(&sem_query),
            kind: SemanticExpressionKind::ImmediateQuery(sem_query),
            ownership: Ownership::Trivial,
        })
    }

    pub(super) fn eval_parameterized_query(
        &mut self,
        parameters: &[TypedQNameNode],
        query: &QueryNode
    ) -> Result<SemanticExpression, SemanticError> {
        let closure_id = self.closure_id_gen.next_id();

        self.enter_scope(SemanticScopeType::Closure(closure_id));
        let mut param_ids = vec![];
        let mut param_types = vec![];
        for param in parameters {
            let sem_type = self.try_get_semantic_type(&param.type_node)?;
            let variable_id = self.variable_id_gen.next_id();
            let closure_scope = self.scopes.last_mut().unwrap();

            closure_scope.variables.insert(param.name.clone(), variable_id);
            self.variables.insert(variable_id, SemanticVariable {
                name: param.name.clone(),
                id: variable_id,
                sem_type: sem_type.clone(),
            });
            param_ids.push(variable_id);
            param_types.push(sem_type);
        }

        self.closures.insert(closure_id, SemanticClosure {
            id: closure_id,
            param_ids,
            captured_variables: vec![],
            return_type: SemanticType::new(SemanticTypeKind::Void),
            body: SemanticClosureBody::dummy()
        });

        let sem_query = self.eval_query(query)?;
        self.exit_scope(false);

        let return_type = self.return_type_of_query(&sem_query);
        let callable_type = SemanticType::new(
            SemanticTypeKind::Callable(param_types, return_type.clone())
        );

        let closure = self.closures.get_mut(&closure_id).unwrap();
        closure.return_type = return_type;
        closure.body = SemanticClosureBody::Query(sem_query);

        Ok(SemanticExpression {
            kind: SemanticExpressionKind::Closure(closure_id),
            sem_type: callable_type,
            ownership: Ownership::Owned,
        })
    }
}