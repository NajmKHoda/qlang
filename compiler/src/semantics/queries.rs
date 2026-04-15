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
    fn eval_qcolumn(&self, qcol: &QColumnNode, select_table_ids: &[u32]) -> Result<SemanticColumn, SemanticError> {
        if let Some(table_name) = &qcol.table_name {
            let table = self.tables.get_by_name(table_name)
                .ok_or_else(|| SemanticError::UndefinedTable {
                    name: table_name.clone()
                })?;
            let table_struct = &self.structs[table.struct_id];
            let column = table_struct.field_index_type(&qcol.column_name);
            return match column {
                Some((col_id, _)) => Ok(SemanticColumn {
                    table_id: table.id,
                    column_index: col_id as u32,
                }),
                None => Err(SemanticError::UndefinedColumn {
                    table_name: table_name.clone(),
                    column_name: qcol.column_name.clone(),
                })
            };
        }

        let mut matching_columns = vec![];
        for table_id in select_table_ids {
            let table = &self.tables[*table_id];
            let table_struct = &self.structs[table.struct_id];
            if let Some((col_id, _)) = table_struct.field_index_type(&qcol.column_name) {
                let already_seen = matching_columns.iter().any(|col: &SemanticColumn| col.table_id == *table_id);
                if !already_seen {
                    matching_columns.push(SemanticColumn {
                        table_id: *table_id,
                        column_index: col_id as u32,
                    });
                }
            }
        }

        match matching_columns.len() {
            0 => Err(SemanticError::SelectUndefinedColumn {
                column_name: qcol.column_name.clone(),
            }),
            1 => Ok(matching_columns.pop().unwrap()),
            _ => Err(SemanticError::SelectAmbiguousColumn {
                column_name: qcol.column_name.clone(),
                table_names: matching_columns
                    .iter()
                    .map(|col| self.tables[col.table_id].name.clone())
                    .collect(),
            }),
        }
    }

    fn eval_where_clause(&self, table: &SemanticTable, column_name: &str, sem_expr: SemanticExpression) -> Result<WhereClause, SemanticError> {
        let table_struct = &self.structs[table.struct_id];
        let column = table_struct.field_index_type(column_name);
        match column {
            Some((col_index, col_type)) => {
                let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                if compatible {
                    Ok(WhereClause {
                        column_index: col_index as u32,
                        value: Box::new(sem_expr),
                    })
                } else {
                    Err(SemanticError::IncompatibleColumnValue {
                        table_name: table.name.clone(),
                        column_name: column_name.to_string(),
                        expected: col_type.clone(),
                        found: sem_expr.sem_type.clone(),
                    })
                }
            },
            None => {
                Err(SemanticError::UndefinedColumn {
                    table_name: table.name.clone(),
                    column_name: column_name.to_string(),
                })
            }
        }        
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

        let mut struct_fields = Vec::new();
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
            struct_fields.push((
                field.name.clone(),
                self.try_get_semantic_type(&field.type_node)?
            ));
        }

        let struct_id = self.struct_id_gen.next_id();
        self.structs.insert(name.to_string(), struct_id, SemanticStruct {
            name: name.to_string(),
            id: struct_id,
            fields: struct_fields,
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
        let table = self.tables.get_by_name(&query.root_table_name)
            .ok_or_else(|| SemanticError::UndefinedTable {
                name: query.root_table_name.clone()
            })?;
        let capturing_struct = self.structs.get_by_name(&query.capturing_struct_name)
            .ok_or_else(|| SemanticError::UndefinedStruct {
                name: query.capturing_struct_name.clone()
            })?;

        let mut table_ids = vec![table.id];
        let mut join_clauses = vec![];
        for join in &query.join_clauses {
            let left_column = self.eval_qcolumn(&join.left_column, &table_ids)?;
            let right_column = self.eval_qcolumn(&join.right_column, &table_ids)?;

            let left_table = &self.tables[left_column.table_id];
            let right_table = &self.tables[right_column.table_id];
            if !table_ids.contains(&left_column.table_id) {
                return Err(SemanticError::InvalidLeftTable {
                    left_table_name: left_table.name.clone()
                });
            }

            let left_struct = &self.structs[left_table.struct_id];
            let right_struct = &self.structs[right_table.struct_id];
            let left_type = &left_struct.fields[left_column.column_index as usize].1;
            let right_type = &right_struct.fields[right_column.column_index as usize].1;
            if !self.try_unify(left_type, right_type) {
                return Err(SemanticError::MismatchingJoinColumnTypes {
                    left_table_name: left_table.name.clone(),
                    left_column_name: join.left_column.column_name.clone(),
                    left_column_type: left_type.clone(),
                    right_table_name: right_table.name.clone(),
                    right_column_name: join.right_column.column_name.clone(),
                    right_column_type: right_type.clone(),
                });
            }

            table_ids.push(right_column.table_id);
            join_clauses.push((left_column, right_column));
        }

        // Check compatibility between capturing struct and captured columns
        let mut captured_columns_map = HashMap::new();
        for (alias, qcol) in &query.captured_columns {
            if captured_columns_map.contains_key(alias) {
                return Err(SemanticError::SelectDuplicateAlias {
                    alias: alias.clone()
                });
            }

            let column = self.eval_qcolumn(qcol, &table_ids)?;
            if !table_ids.contains(&column.table_id) {
                return Err(SemanticError::ExcludedTableInSelect {
                    table_name: self.tables[column.table_id].name.clone()
                });
            }
            captured_columns_map.insert(alias.clone(), column);
        }

        if captured_columns_map.len() != capturing_struct.fields.len() {
            return Err(SemanticError::SelectIncompatibleCapture {
                capturing_struct: capturing_struct.name.clone(),
            });
        }

        let mut captured_columns = vec![];
        for (field_name, field_type) in &capturing_struct.fields {
            match captured_columns_map.get(field_name) {
                Some(SemanticColumn { table_id, column_index }) => {
                    let table = &self.tables[*table_id];
                    let table_struct = &self.structs[table.struct_id];
                    let (_, col_type) = &table_struct.fields[*column_index as usize];
                    let compatible = self.try_downcast(field_type, col_type);
                    if !compatible {
                        return Err(SemanticError::SelectIncompatibleCapture {
                            capturing_struct: capturing_struct.name.clone(),
                        });
                    }
                    captured_columns.push(SemanticColumn {
                        table_id: *table_id,
                        column_index: *column_index,
                    });
                },
                None => {
                    return Err(SemanticError::SelectIncompatibleCapture {
                        capturing_struct: capturing_struct.name.clone(),
                    });
                }
            }
        }
        
        // TODO: Reimplement where clause
        /*
        let where_expr = query.where_clause.as_ref().map(|where_node| {
            let sem_expr = self.eval_expr(&where_node.value)?;
            Ok((where_node.column_name.clone(), sem_expr))
         }).transpose()?;

        let where_clause = match query.where_clause {
            Some(where_node) => Some(self.eval_where_clause(table, &column_name, sem_expr)?),
            None => None,
        };
        */

        Ok(SemanticQuery::Select {
            capturing_struct_id: capturing_struct.id,
            captured_columns,
            root_table_id: table.id,
            join_clauses,
            where_clause: None,
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
            let column = table_struct.field_index_type(col_name);
            match column {
                Some((col_index, col_type)) => {
                    let compatible = self.try_downcast(col_type, &sem_expr.sem_type);
                    if compatible {
                        Ok(UpdateAssignment {
                            column_index: col_index as u32,
                            value: sem_expr,
                        })
                    } else {
                        return Err(SemanticError::IncompatibleColumnValue {
                            table_name: table.name.clone(),
                            column_name: col_name.to_string(),
                            expected: col_type.clone(),
                            found: sem_expr.sem_type.clone(),
                        });
                    }
                },
                None => {
                    Err(SemanticError::UndefinedColumn {
                        table_name: table.name.clone(),
                        column_name: col_name.to_string(),
                    })
                }
            }
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
            SemanticQuery::Select { capturing_struct_id, .. } => {
                let _struct = &self.structs[*capturing_struct_id];
                let struct_type = SemanticType::new(SemanticTypeKind::NamedStruct(
                    *capturing_struct_id,
                    _struct.name.clone(),
                ));
                SemanticType::new(SemanticTypeKind::Iterator(struct_type))
            },
            _ => SemanticType::new(SemanticTypeKind::Void),
        }
    }

    pub(super) fn eval_immediate_query(&mut self, query: &QueryNode) -> Result<SemanticExpression, SemanticError> {
        let sem_query = self.eval_query(query)?;
        let return_type = self.return_type_of_query(&sem_query);

        Ok(SemanticExpression {
            ownership: if return_type.can_be_owned() {
                Ownership::Trivial
            } else {
                Ownership::Owned
            },
            sem_type: return_type,
            kind: SemanticExpressionKind::ImmediateQuery(sem_query),
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