use inkwell::values::{AnyValue, BasicValue, PointerValue};

use crate::{codegen::{data::GenValue}, semantics::{Ownership, SelectCountClause, SemanticQuery, SemanticType, SemanticTypeKind, SelectWhereClause, WhereClause}};

use super::{CodeGen, CodeGenError};

pub(super) enum ColumnType {
    Integer,
    Bool,
    String,
    Real,
}

impl From<&SemanticType> for ColumnType {
    fn from(sem_type: &SemanticType) -> Self {
        match sem_type.kind() {
            SemanticTypeKind::Integer => ColumnType::Integer,
            SemanticTypeKind::Float => ColumnType::Real,
            SemanticTypeKind::Bool => ColumnType::Bool,
            SemanticTypeKind::String => ColumnType::String,
            _ => panic!("Unsupported column type in semantic IR"),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
	pub(super) fn gen_immediate_query(&mut self, query: &SemanticQuery, error_drops: &[u32]) -> Result<GenValue<'ctxt>, CodeGenError> {
        let prepared_stmt = self.prepare_query(query)?;
        let prep_failed = self.builder.build_is_null(prepared_stmt, "query_prepare_failed")?;
        let (result, exec_ok) = self.execute_query(prepared_stmt, query)?;
        let exec_failed = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            exec_ok,
            self.bool_type().const_int(0, false),
            "query_exec_failed",
        )?;
        let is_error = self.builder.build_or(prep_failed, exec_failed, "query_is_error")?;
		self.gen_failable_check(is_error, error_drops)?;
        self.finalize_query(prepared_stmt, query)?;
        Ok(result)
    }

    pub(super) fn prepare_query(&mut self, query: &SemanticQuery) -> Result<PointerValue<'ctxt>, CodeGenError> {
        match query {
            SemanticQuery::Select { select_table_ids, capturing_struct_id, captured_columns, join_clauses, where_clause, limit_clause, offset_clause } => {
                let root_table_id = select_table_ids[0];
                let table_info = &self.table_info[&root_table_id];
                let struct_info = &self.struct_info[&capturing_struct_id];
                let select_plan_ptr = self.builder.build_call(
                    self.runtime.select_plan_new,
                    &[
                        table_info.name_str.as_pointer_value().into(),
                        self.int_type().const_int(captured_columns.len() as u64, false).into(),
                        self.int_type().const_int(join_clauses.len() as u64, false).into(),
                        struct_info.type_info.as_pointer_value().into(),
                    ],
                    "select_plan"
                )?.as_any_value_enum().into_pointer_value();

                for (i, column) in captured_columns.iter().enumerate() {
                    let table_id = select_table_ids[column.table_index as usize];
                    let table_info = &self.table_info[&table_id];
                    let column_name_str = table_info.column_name_strs[column.column_index as usize];
                    self.builder.build_call(
                        self.runtime.select_plan_set_column,
                        &[
                            select_plan_ptr.into(),
                            self.context.i32_type().const_int(i as u64, false).into(),
                            self.context.i32_type().const_int(column.table_index as u64, false).into(),
                            column_name_str.as_pointer_value().into(),
                        ],
                        &format!("select_plan_set_column_{}", i)
                    )?;
                }

                for (i, (left_col, right_col)) in join_clauses.iter().enumerate() {
                    let left_table_id = select_table_ids[left_col.table_index as usize];
                    let right_table_id = select_table_ids[right_col.table_index as usize];
                    let left_table_info = &self.table_info[&left_table_id];
                    let right_table_info = &self.table_info[&right_table_id];
                    let right_table_name_str = right_table_info.name_str;
                    let left_column_name_str = left_table_info.column_name_strs[left_col.column_index as usize];
                    let right_column_name_str = right_table_info.column_name_strs[right_col.column_index as usize];
                    self.builder.build_call(
                        self.runtime.select_plan_set_join,
                        &[
                            select_plan_ptr.into(),
                            self.context.i32_type().const_int(i as u64, false).into(),
                            right_table_name_str.as_pointer_value().into(),
                            self.context.i32_type().const_int(left_col.table_index as u64, false).into(),
                            left_column_name_str.as_pointer_value().into(),
                            self.context.i32_type().const_int(right_col.table_index as u64, false).into(),
                            right_column_name_str.as_pointer_value().into(),
                        ],
                        &format!("select_plan_set_join_{}", i)
                    )?;
                }

                if let Some(SelectWhereClause { column, .. }) = where_clause {
                    let where_table_id = select_table_ids[column.table_index as usize];
                    let where_table_info = &self.table_info[&where_table_id];
                    let column_name_str = where_table_info.column_name_strs[column.column_index as usize];
                    self.builder.build_call(
                        self.runtime.select_plan_set_where,
                        &[
                            select_plan_ptr.into(),
                            self.context.i32_type().const_int(column.table_index as u64, false).into(),
                            column_name_str.as_pointer_value().into(),
                        ],
                        "select_plan_set_where"
                    )?;
                }

                if limit_clause.is_some() {
                    self.builder.build_call(
                        self.runtime.select_plan_set_limit,
                        &[select_plan_ptr.into()],
                        "select_plan_set_limit"
                    )?;
                }

                if offset_clause.is_some() {
                    self.builder.build_call(
                        self.runtime.select_plan_set_offset,
                        &[select_plan_ptr.into()],
                        "select_plan_set_offset"
                    )?;
                }

                let select_iterator = self.builder.build_call(
                    self.runtime.select_plan_prepare,
                    &[select_plan_ptr.into()],
                    "prepare_select"
                )?.as_any_value_enum().into_pointer_value();

                Ok(select_iterator)
            },
            SemanticQuery::Insert { table_id, .. } => {
                let table = &self.program.tables[table_id];
                let table_info = &self.table_info[table_id];
                let struct_info = &self.struct_info[&table.struct_id];
                let insert_plan_ptr = self.builder.build_call(
                    self.runtime.insert_plan_new,
                    &[
                        table_info.name_str.as_pointer_value().into(),
                        struct_info.type_info.as_pointer_value().into(),
                    ],
                    "insert_plan"
                )?.as_any_value_enum().into_pointer_value();

                let prepared_insert = self.builder.build_call(
                    self.runtime.insert_plan_prepare,
                    &[insert_plan_ptr.into()],
                    "prepared_insert"
                )?.as_any_value_enum().into_pointer_value();

                Ok(prepared_insert)
            }
            SemanticQuery::Update { table_id, assignments, where_clause } => {
                let table_info = &self.table_info[table_id];
                let col_name_arr_type = self.ptr_type().array_type(assignments.len() as u32);
                let col_name_arr = self.build_alloca(col_name_arr_type.into(), "col_name_arr")?;
                for (i, assignment) in assignments.iter().enumerate() {
                    let col_index = assignment.column_index as usize;
                    let column_name_str = table_info.column_name_strs[col_index]
                        .as_basic_value_enum();
                    let elem_ptr = unsafe { self.builder.build_gep(
                        col_name_arr_type,
                        col_name_arr,
                        &[
                            self.context.i32_type().const_zero(),
                            self.context.i32_type().const_int(i as u64, false)
                        ],
                        &format!("col_name_ptr_{}", i)
                    )? };
                    self.builder.build_store(elem_ptr, column_name_str)?;
                }

                let update_plan_ptr = self.builder.build_call(
                    self.runtime.update_plan_new,
                    &[
                        table_info.name_str.as_pointer_value().into(),
                        self.int_type().const_int(assignments.len() as u64, false).into(),
                        col_name_arr.into(),
                    ],
                    "update_plan"
                )?.as_any_value_enum().into_pointer_value();

                if let Some(WhereClause { column_index, .. }) = where_clause {
                    let column_name_str = table_info.column_name_strs[*column_index as usize];
                    self.builder.build_call(
                        self.runtime.update_plan_set_where,
                        &[
                            update_plan_ptr.into(),
                            column_name_str.as_pointer_value().into(),
                        ],
                        "update_plan_set_where"
                    )?;
                }

                let prepared_update = self.builder.build_call(
                    self.runtime.update_plan_prepare,
                    &[update_plan_ptr.into()],
                    "prepared_update"
                )?.as_any_value_enum().into_pointer_value();

                Ok(prepared_update)
            }
            SemanticQuery::Delete { table_id, where_clause } => {
                let table = &self.program.tables[table_id];
                let table_info = &self.table_info[table_id];
                let struct_info = &self.struct_info[&table.struct_id];
                let select_plan_ptr = self.builder.build_call(
                    self.runtime.delete_plan_new,
                    &[
                        table_info.name_str.as_pointer_value().into(),
                        struct_info.type_info.as_pointer_value().into(),
                    ],
                    "delete_plan"
                )?.as_any_value_enum().into_pointer_value();

                if let Some(WhereClause { column_index, .. }) = where_clause {
                    let column_name_str = table_info.column_name_strs[*column_index as usize];
                    self.builder.build_call(
                        self.runtime.delete_plan_set_where,
                        &[
                            select_plan_ptr.into(),
                            column_name_str.as_pointer_value().into(),
                        ],
                        "delete_plan_set_where"
                    )?;
                }

                let prepared_delete = self.builder.build_call(
                    self.runtime.delete_plan_prepare,
                    &[select_plan_ptr.into()],
                    "prepared_delete"
                )?.as_any_value_enum().into_pointer_value();

                Ok(prepared_delete)
            }
        }
    }

    pub(super) fn execute_query(
        &mut self,
        statement: PointerValue<'ctxt>,
        query: &SemanticQuery
    ) -> Result<(GenValue<'ctxt>, inkwell::values::IntValue<'ctxt>), CodeGenError> {
        match query {
            SemanticQuery::Select { where_clause, limit_clause, offset_clause, select_table_ids, .. } => {
                let select_iterator = self.builder.build_call(
                    self.runtime.select_iterator_activate,
                    &[statement.into()],
                    "activate_select_iterator"
                )?.as_any_value_enum().into_pointer_value();
                let mut ok_flag = self.builder.build_is_not_null(select_iterator, "select_activate_ok")?;

                if let Some(SelectWhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "select_where")?;
                    let column_type: ColumnType = (&value.sem_type).into();
                    let column_type_int = self.context.i32_type().const_int(column_type as u64, false);
                    let where_ok = self.builder.build_call(
                        self.runtime.select_iterator_bind_where,
                        &[
                            select_iterator.into(),
                            column_type_int.into(),
                            value_ptr.into(),
                        ],
                        "select_bind_where"
                    )?.as_any_value_enum().into_int_value();
                    ok_flag = self.builder.build_and(ok_flag, where_ok, "select_ok_with_where")?;
                }

                if let Some(SelectCountClause { value }) = limit_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "select_limit")?;
                    let limit_ok = self.builder.build_call(
                        self.runtime.select_iterator_bind_limit,
                        &[select_iterator.into(), value_ptr.into()],
                        "select_bind_limit"
                    )?.as_any_value_enum().into_int_value();
                    ok_flag = self.builder.build_and(ok_flag, limit_ok, "select_ok_with_limit")?;
                }

                if let Some(SelectCountClause { value }) = offset_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "select_offset")?;
                    let offset_ok = self.builder.build_call(
                        self.runtime.select_iterator_bind_offset,
                        &[select_iterator.into(), value_ptr.into()],
                        "select_bind_offset"
                    )?.as_any_value_enum().into_int_value();
                    ok_flag = self.builder.build_and(ok_flag, offset_ok, "select_ok_with_offset")?;
                }

                let table = &self.program.tables[&select_table_ids[0]];
                let elem_type = SemanticType::new(
                    SemanticTypeKind::NamedStruct(table.struct_id, table.name.clone())
                );
                Ok((GenValue::Iterator {
                    value: select_iterator,
                    elem_type,
                    ownership: Ownership::Borrowed,
                }, ok_flag))
            },
            SemanticQuery::Insert { value: insert_value, .. } => {
                let gen_value = self.gen_eval(insert_value)?;
                let success = match gen_value {
                    GenValue::Array { value: llvm_value, .. } => {
                        self.builder.build_call(
                            self.runtime.prepared_insert_exec_array.into(),
                            &[statement.into(), llvm_value.into()],
                            "insert_exec_array"
                        )?.as_any_value_enum().into_int_value()
                    }
                    GenValue::Struct { .. } => {
                        let data_ptr = self.put_on_stack(&gen_value, "insert_row")?;
                        self.builder.build_call(
                            self.runtime.prepared_insert_exec_row.into(),
                            &[statement.into(), data_ptr.into()],
                            "insert_exec_row"
                        )?.as_any_value_enum().into_int_value()
                    }
                    _ => panic!("Unexpected insert value type")
                };
                Ok((GenValue::Void, success))
            },
            SemanticQuery::Update { assignments, where_clause, .. } => {
                let mut success = self.bool_type().const_int(1, false);
                for (i, assignment) in assignments.iter().enumerate() {
                    let gen_value = self.gen_eval(&assignment.value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "update_assign")?;
                    let column_type: ColumnType = (&assignment.value.sem_type).into();
                    let column_type_int = self.context.i32_type().const_int(column_type as u64, false);
                    let bind_ok = self.builder.build_call(
                        self.runtime.prepared_update_bind_assignment,
                        &[
                            statement.into(),
                            self.context.i32_type().const_int(i as u64, false).into(),
                            column_type_int.into(),
                            value_ptr.into(),
                        ],
                        &format!("update_bind_assign_{}", i)
                    )?.as_any_value_enum().into_int_value();
                    success = self.builder.build_and(success, bind_ok, "update_bind_ok")?;
                }

                if let Some(WhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "update_where")?;
                    let column_type: ColumnType = (&value.sem_type).into();
                    let column_type_int = self.context.i32_type().const_int(column_type as u64, false);
                    let where_ok = self.builder.build_call(
                        self.runtime.prepared_update_bind_where,
                        &[
                            statement.into(),
                            column_type_int.into(),
                            value_ptr.into(),
                        ],
                        "update_bind_where"
                    )?.as_any_value_enum().into_int_value();
                    success = self.builder.build_and(success, where_ok, "update_where_ok")?;
                }

                let exec_ok = self.builder.build_call(
                    self.runtime.prepared_update_exec.into(),
                    &[statement.into()],
                    "execute_update"
                )?.as_any_value_enum().into_int_value();
                success = self.builder.build_and(success, exec_ok, "update_exec_ok")?;
                Ok((GenValue::Void, success))
            },
            SemanticQuery::Delete { where_clause, .. } => {
                let mut success = self.bool_type().const_int(1, false);
                if let Some(WhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.put_on_stack(&gen_value, "delete_where")?;
                    let column_type: ColumnType = (&value.sem_type).into();
                    let column_type_int = self.context.i32_type().const_int(column_type as u64, false);
                    let where_ok = self.builder.build_call(
                        self.runtime.prepared_delete_bind_where,
                        &[
                            statement.into(),
                            column_type_int.into(),
                            value_ptr.into(),
                        ],
                        "delete_bind_where"
                    )?.as_any_value_enum().into_int_value();
                    success = self.builder.build_and(success, where_ok, "delete_where_ok")?;
                }

                let exec_ok = self.builder.build_call(
                    self.runtime.prepared_delete_exec.into(),
                    &[statement.into()],
                    "execute_delete"
                )?.as_any_value_enum().into_int_value();
                success = self.builder.build_and(success, exec_ok, "delete_exec_ok")?;

                Ok((GenValue::Void, success))
            }
        }
    }

    pub(super) fn finalize_query(
        &self,
        statement: PointerValue<'ctxt>,
        query: &SemanticQuery
    ) -> Result<(), CodeGenError> {
        match query {
            SemanticQuery::Insert { .. } => {
                self.builder.build_call(
                    self.runtime.prepared_insert_finalize.into(),
                    &[statement.into()],
                    "finalize_insert"
                )?;
            },
            SemanticQuery::Update { .. } => {
                self.builder.build_call(
                    self.runtime.prepared_update_finalize.into(),
                    &[statement.into()],
                    "finalize_update"
                )?;
            },
            SemanticQuery::Delete { .. } => {
                self.builder.build_call(
                    self.runtime.prepared_delete_finalize.into(),
                    &[statement.into()],
                    "finalize_delete"
                )?;
            }

            // Iterators have their own management
            SemanticQuery::Select { .. } => {},
        }
        Ok(())
    }
}