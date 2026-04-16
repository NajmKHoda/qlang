use super::*;

pub struct SemanticDatasource {
    #[allow(dead_code)]
    pub name: String,
    pub id: u32,
    pub is_readonly: bool,
}

pub struct SemanticTable {
    pub name: String,
    pub id: u32,
    pub is_readonly: bool,
    pub struct_id: u32,
    #[allow(dead_code)]
    pub datasource_id: u32,
}

impl SemanticGen {
    fn eval_qcolumn(
        &self,
        qcol: &QColumnNode,
        select_table_ids: &[u32],
        select_alias_map: &HashMap<String, u32>,
    ) -> Result<SemanticColumn, SemanticError> {
        if let Some(table_alias) = &qcol.table_name {
            let Some(&table_index) = select_alias_map.get(table_alias) else {
                return Err(SemanticError::SelectUndefinedTableAlias {
                    alias: table_alias.clone(),
                });
            };

            let table_id = select_table_ids[table_index as usize];
            let table = &self.tables[table_id];
            let table_struct = &self.structs[table.struct_id];
            let column = table_struct.field_index_type(&qcol.column_name);
            return match column {
                Some((col_id, _)) => Ok(SemanticColumn {
                    table_index,
                    column_index: col_id as u32,
                }),
                None => Err(SemanticError::UndefinedColumn {
                    table_name: table.name.clone(),
                    column_name: qcol.column_name.clone(),
                })
            };
        }

        let mut matching_columns = vec![];
        for (table_index, table_id) in select_table_ids.iter().enumerate() {
            let table = &self.tables[*table_id];
            let table_struct = &self.structs[table.struct_id];
            if let Some((col_id, _)) = table_struct.field_index_type(&qcol.column_name) {
                matching_columns.push(SemanticColumn {
                    table_index: table_index as u32,
                    column_index: col_id as u32,
                });
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
                    .map(|col| {
                        let table_id = select_table_ids[col.table_index as usize];
                        self.tables[table_id].name.clone()
                    })
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

    fn eval_select_where_clause(
        &self,
        qcol: &QColumnNode,
        sem_expr: SemanticExpression,
        select_table_ids: &[u32],
        select_alias_map: &HashMap<String, u32>,
    ) -> Result<SelectWhereClause, SemanticError> {
        let column = self.eval_qcolumn(qcol, select_table_ids, select_alias_map)?;
        let table_id = select_table_ids[column.table_index as usize];
        let table = &self.tables[table_id];
        let table_struct = &self.structs[table.struct_id];
        let (_, col_type) = &table_struct.fields[column.column_index as usize];
        if !self.try_downcast(col_type, &sem_expr.sem_type) {
            return Err(SemanticError::IncompatibleColumnValue {
                table_name: table.name.clone(),
                column_name: qcol.column_name.clone(),
                expected: col_type.clone(),
                found: sem_expr.sem_type,
            });
        }

        Ok(SelectWhereClause {
            column,
            value: Box::new(sem_expr),
        })
    }

    fn eval_select_count_clause(
        &self,
        sem_expr: SemanticExpression,
        clause_name: &'static str,
    ) -> Result<SelectCountClause, SemanticError> {
        let int_type = SemanticType::new(SemanticTypeKind::Integer);
        if !self.try_downcast(&int_type, &sem_expr.sem_type) {
            return Err(SemanticError::SelectNonIntegralCountClause {
                clause_name,
                found_type: sem_expr.sem_type,
            });
        }

        Ok(SelectCountClause {
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

        let mut struct_fields = Vec::new();
        for field in fields {
            let is_primitive = match field.type_node {
                TypeNode::Integer | TypeNode::Float | TypeNode::Bool | TypeNode::String => true,
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

        let mut select_table_ids = vec![table.id];
        let root_alias = match &query.root_table_alias {
            Some(alias) => alias.clone(),
            None => query.root_table_name.clone(),
        };
        let mut select_alias_map = HashMap::new();
        select_alias_map.insert(root_alias.clone(), 0);

        let (capturing_struct_id, capturing_struct_name, capturing_struct_fields, captured_columns_node) =
            match &query.capture {
                SelectCaptureNode::Explicit {
                    capturing_struct_name,
                    captured_columns,
                } => {
                    let capturing_struct = self.structs.get_by_name(capturing_struct_name)
                        .ok_or_else(|| SemanticError::UndefinedStruct {
                            name: capturing_struct_name.clone(),
                        })?;

                    (
                        capturing_struct.id,
                        capturing_struct.name.clone(),
                        capturing_struct.fields.clone(),
                        captured_columns.iter().map(|(alias, qcol)| {
                            (
                                alias.clone(),
                                QColumnNode {
                                    table_name: qcol.table_name.clone(),
                                    column_name: qcol.column_name.clone(),
                                },
                            )
                        }).collect::<Vec<_>>(),
                    )
                }
                SelectCaptureNode::All => {
                    let table_struct = &self.structs[table.struct_id];
                    (
                        table_struct.id,
                        table_struct.name.clone(),
                        table_struct.fields.clone(),
                        table_struct.fields.iter().map(|(field_name, _)| {
                            (
                                field_name.clone(),
                                QColumnNode {
                                    table_name: Some(root_alias.clone()),
                                    column_name: field_name.clone(),
                                },
                            )
                        }).collect::<Vec<_>>(),
                    )
                }
            };

        let mut join_clauses = vec![];
        for join in &query.join_clauses {
            let right_table = self.tables.get_by_name(&join.right_table_name)
                .ok_or_else(|| SemanticError::UndefinedTable {
                    name: join.right_table_name.clone()
                })?;
            let right_alias = join.right_table_alias
                .clone()
                .unwrap_or_else(|| join.right_table_name.clone());
            if select_alias_map.contains_key(&right_alias) {
                return Err(SemanticError::SelectDuplicateTableAlias {
                    alias: right_alias,
                });
            }

            let right_table_index = select_table_ids.len() as u32;
            select_alias_map.insert(right_alias, right_table_index);
            select_table_ids.push(right_table.id);

            let left_column = self.eval_qcolumn(&join.left_column, &select_table_ids, &select_alias_map)?;

            let right_struct = &self.structs[right_table.struct_id];
            let Some((right_col_id, _)) = right_struct.field_index_type(&join.right_column_name) else {
                return Err(SemanticError::UndefinedColumn {
                    table_name: right_table.name.clone(),
                    column_name: join.right_column_name.clone(),
                });
            };
            let right_column = SemanticColumn {
                table_index: right_table_index,
                column_index: right_col_id as u32,
            };

            let left_table = &self.tables[select_table_ids[left_column.table_index as usize]];

            let left_struct = &self.structs[left_table.struct_id];
            let left_type = &left_struct.fields[left_column.column_index as usize].1;
            let right_type = &right_struct.fields[right_column.column_index as usize].1;
            if !self.try_unify(left_type, right_type) {
                return Err(SemanticError::MismatchingJoinColumnTypes {
                    left_table_name: left_table.name.clone(),
                    left_column_name: join.left_column.column_name.clone(),
                    left_column_type: left_type.clone(),
                    right_table_name: right_table.name.clone(),
                    right_column_name: join.right_column_name.clone(),
                    right_column_type: right_type.clone(),
                });
            }

            join_clauses.push((left_column, right_column));
        }

        // Check compatibility between capturing struct and captured columns
        let mut captured_columns_map = HashMap::new();
        for (alias, qcol) in &captured_columns_node {
            if captured_columns_map.contains_key(alias) {
                return Err(SemanticError::SelectDuplicateAlias {
                    alias: alias.clone()
                });
            }

            let column = self.eval_qcolumn(qcol, &select_table_ids, &select_alias_map)?;
            captured_columns_map.insert(alias.clone(), column);
        }

        if captured_columns_map.len() != capturing_struct_fields.len() {
            return Err(SemanticError::SelectIncompatibleCapture {
                capturing_struct: capturing_struct_name.clone(),
            });
        }

        let mut captured_columns = vec![];
        for (field_name, field_type) in &capturing_struct_fields {
            match captured_columns_map.get(field_name) {
                Some(SemanticColumn { table_index, column_index }) => {
                    let table = &self.tables[select_table_ids[*table_index as usize]];
                    let table_struct = &self.structs[table.struct_id];
                    let (_, col_type) = &table_struct.fields[*column_index as usize];
                    let compatible = self.try_downcast(field_type, col_type);
                    if !compatible {
                        return Err(SemanticError::SelectIncompatibleCapture {
                            capturing_struct: capturing_struct_name.clone(),
                        });
                    }
                    captured_columns.push(SemanticColumn {
                        table_index: *table_index,
                        column_index: *column_index,
                    });
                },
                None => {
                    return Err(SemanticError::SelectIncompatibleCapture {
                        capturing_struct: capturing_struct_name.clone(),
                    });
                }
            }
        }
        
        let where_clause = query.where_clause.as_ref().map(|where_node| {
            let sem_expr = self.eval_expr(&where_node.value)?;
            self.eval_select_where_clause(
                &where_node.column,
                sem_expr,
                &select_table_ids,
                &select_alias_map,
            )
        }).transpose()?;

        let limit_clause = query.limit_clause.as_ref().map(|limit_node| {
            let sem_expr = self.eval_expr(&limit_node.value)?;
            self.eval_select_count_clause(sem_expr, "LIMIT")
        }).transpose()?;

        let offset_clause = query.offset_clause.as_ref().map(|offset_node| {
            let sem_expr = self.eval_expr(&offset_node.value)?;
            self.eval_select_count_clause(sem_expr, "OFFSET")
        }).transpose()?;

        Ok(SemanticQuery::Select {
            capturing_struct_id,
            captured_columns,
            select_table_ids,
            join_clauses,
            where_clause,
            limit_clause,
            offset_clause,
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
        if !self.cur_environment_is_failable() {
            return Err(SemanticError::QueryInNonFailableFunction {
                function_name: self.cur_executable_name(),
            });
        }

        let sem_query = self.eval_query(query)?;
        let return_type = self.return_type_of_query(&sem_query);

        Ok(SemanticExpression {
            ownership: if return_type.can_be_owned() {
                Ownership::Trivial
            } else {
                Ownership::Owned
            },
            sem_type: return_type,
            kind: SemanticExpressionKind::ImmediateQuery {
                query: sem_query,
                error_drops: self.cur_error_drops(),
            },
        })
    }

    pub(super) fn eval_parameterized_query(
        &mut self,
        parameters: &[TypedQNameNode],
        query: &QueryNode
    ) -> Result<SemanticExpression, SemanticError> {
        if !self.cur_environment_is_failable() {
            return Err(SemanticError::QueryInNonFailableFunction {
                function_name: self.cur_executable_name(),
            });
        }

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
            is_failable: true,
            param_ids,
            captured_variables: vec![],
            return_type: SemanticType::new(SemanticTypeKind::Void),
            body: SemanticClosureBody::dummy()
        });

        let sem_query = self.eval_query(query)?;
        self.exit_scope(false);

        let return_type = self.return_type_of_query(&sem_query);
        let callable_type = SemanticType::new(
            SemanticTypeKind::Callable {
                is_failable: true,
                param_types,
                ret_type: return_type.clone(),
            }
        );

        let closure = self.closures.get_mut(&closure_id).unwrap();
        closure.return_type = return_type;
        closure.body = SemanticClosureBody::Query(sem_query);

        Ok(SemanticExpression {
            kind: SemanticExpressionKind::Closure {
                closure_id,
                error_drops: self.cur_error_drops(),
            },
            sem_type: callable_type,
            ownership: Ownership::Owned,
        })
    }
}