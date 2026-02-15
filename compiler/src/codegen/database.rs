use inkwell::{AddressSpace, values::{AnyValue, BasicValue, FunctionValue, PointerValue}};

use crate::{codegen::{data::GenValue}, semantics::{Ownership, SemanticDatasource, SemanticQuery, SemanticType, SemanticTypeKind, WhereClause}};

use super::{CodeGen, CodeGenError};

impl<'ctxt> CodeGen<'ctxt> {
    fn place_onto_stack(
        &mut self,
        value: &GenValue<'ctxt>,
    ) -> Result<PointerValue<'ctxt>, CodeGenError> {
        let llvm_value = value.as_llvm_basic_value();
        let alloca = self.builder.build_alloca(llvm_value.get_type(), "stack_alloca")?;
        self.builder.build_store(alloca, llvm_value)?;
        Ok(alloca)
    }

    pub(super) fn gen_database_ptr(&mut self, datasource: &SemanticDatasource) {
        let db_ptr_global = self.module.add_global(
            self.ptr_type(),
            Some(AddressSpace::default()),
            format!("{}_ptr", datasource.name).as_str()
        );
        db_ptr_global.set_initializer(&self.ptr_type().const_null());
        self.datasource_ptrs.insert(datasource.id, db_ptr_global.as_pointer_value());
    }

    pub(super) fn init_databases(
        &mut self,
        main_fn: FunctionValue<'ctxt>
    ) -> Result<PointerValue<'ctxt>, CodeGenError> {
        // Grab command line arguments
        let argc = main_fn.get_nth_param(0).unwrap().into_int_value();
        let argv = main_fn.get_nth_param(1).unwrap().into_pointer_value();

        // Create an array of global database pointers to feed to the runtime function
        let num_dbs = self.program.datasources.len() as u32;
        let db_ptr_arr_type = self.ptr_type().array_type(num_dbs);
        let db_ptr_arr = self.builder.build_alloca(db_ptr_arr_type, "db_ptr_arr")?;
        for (i, datasource) in self.program.datasources.values().enumerate() {
            let db_ptr = self.datasource_ptrs[&datasource.id];
            let index = self.context.i32_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    db_ptr_arr_type,
                    db_ptr_arr,
                    &[self.context.i32_type().const_zero(), index],
                    &format!("db_ptr_{}", i)
                )?
            };
            self.builder.build_store(elem_ptr, db_ptr)?;
        }

        // Call into the runtime to initialize databases
        self.builder.build_call(
            self.runtime.init_dbs.into(),
            &[
                argc.into(),
                argv.into(),
                self.context.i32_type().const_int(num_dbs as u64, false).into(),
                db_ptr_arr.into(),
            ],
            "init_dbs_call"
        )?;
            
        Ok(db_ptr_arr)
    }

    pub(super) fn close_databases(&self, db_ptr_arr: PointerValue<'ctxt>) -> Result<(), CodeGenError> {
        let num_dbs = self.datasource_ptrs.len() as u32;
        self.builder.build_call(
            self.runtime.close_dbs.into(),
            &[
                self.context.i32_type().const_int(num_dbs as u64, false).into(),
                db_ptr_arr.into(),
            ],
            "close_dbs_call"
        )?;
        Ok(())
    }

    pub(super) fn gen_immediate_query(&mut self, query: &SemanticQuery) -> Result<GenValue<'ctxt>, CodeGenError> {
        let prepared_stmt = self.prepare_query(query)?;
        let result = self.execute_query(prepared_stmt, query)?;
        self.finalize_query(prepared_stmt, query)?;
        Ok(result)
    }

    pub(super) fn prepare_query(&mut self, query: &SemanticQuery) -> Result<PointerValue<'ctxt>, CodeGenError> {
        match query {
            SemanticQuery::Select { table_id, where_clause } => {
                let table = &self.program.tables[table_id];
                let table_info = &self.table_info[table_id];
                let struct_info = &self.struct_info[&table.struct_id];
                let select_plan_ptr = self.builder.build_call(
                    self.runtime.select_plan_new,
                    &[
                        table_info.name_str.as_pointer_value().into(),
                        struct_info.type_info.as_pointer_value().into(),
                    ],
                    "select_plan"
                )?.as_any_value_enum().into_pointer_value();

                if let Some(WhereClause { column_index, .. }) = where_clause {
                    let column_name_str = table_info.column_name_strs[*column_index as usize];
                    self.builder.build_call(
                        self.runtime.select_plan_set_where,
                        &[
                            select_plan_ptr.into(),
                            column_name_str.as_pointer_value().into(),
                        ],
                        "select_plan_set_where"
                    )?;
                }

                let database_global = self.datasource_ptrs[&table.datasource_id];
                let database_ptr = self.builder.build_load(
                    self.ptr_type(),
                    database_global,
                    "load_database_ptr"
                )?.into_pointer_value();
                let prepared_select = self.builder.build_call(
                    self.runtime.select_plan_prepare,
                    &[database_ptr.into(), select_plan_ptr.into()],
                    "prepared_select"
                )?.as_any_value_enum().into_pointer_value();

                Ok(prepared_select)
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

                let database_global = self.datasource_ptrs[&table.datasource_id];
                let database_ptr = self.builder.build_load(
                    self.ptr_type(),
                    database_global,
                    "load_database_ptr"
                )?.into_pointer_value();
                let prepared_insert = self.builder.build_call(
                    self.runtime.insert_plan_prepare,
                    &[database_ptr.into(), insert_plan_ptr.into()],
                    "prepared_insert"
                )?.as_any_value_enum().into_pointer_value();

                Ok(prepared_insert)
            }
            SemanticQuery::Update { table_id, assignments, where_clause } => {
                let table = &self.program.tables[table_id];
                let table_info = &self.table_info[table_id];

                let col_name_arr_type = self.ptr_type().array_type(assignments.len() as u32);
                let col_name_arr = self.builder.build_alloca(col_name_arr_type, "col_name_arr")?;
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

                let database_global = self.datasource_ptrs[&table.datasource_id];
                let database_ptr = self.builder.build_load(
                    self.ptr_type(),
                    database_global,
                    "load_database_ptr"
                )?.into_pointer_value();
                let prepared_update = self.builder.build_call(
                    self.runtime.update_plan_prepare,
                    &[database_ptr.into(), update_plan_ptr.into()],
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

                let database_global = self.datasource_ptrs[&table.datasource_id];
                let database_ptr = self.builder.build_load(
                    self.ptr_type(),
                    database_global,
                    "load_database_ptr"
                )?.into_pointer_value();
                let prepared_delete = self.builder.build_call(
                    self.runtime.delete_plan_prepare,
                    &[database_ptr.into(), select_plan_ptr.into()],
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
    ) -> Result<GenValue<'ctxt>, CodeGenError> {
        match query {
            SemanticQuery::Select { where_clause, table_id } => {
                if let Some(WhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.place_onto_stack(&gen_value)?;
                    self.builder.build_call(
                        self.runtime.prepared_select_bind_where,
                        &[
                            statement.into(),
                            self.get_qltype(&value.sem_type).into(),
                            value_ptr.into(),
                        ],
                        "select_bind_where"
                    )?;
                }
                let result = self.builder.build_call(
                    self.runtime.prepared_select_execute.into(),
                    &[statement.into()],
                    "execute_select"
                )?.as_any_value_enum().into_pointer_value();

                let table = &self.program.tables[table_id];
                let elem_type = SemanticType::new(
                    SemanticTypeKind::NamedStruct(table.struct_id, table.name.clone())
                );
                Ok(GenValue::Array {
                    value: result,
                    elem_type,
                    ownership: Ownership::Owned,
                })
            },
            SemanticQuery::Insert { value: insert_value, .. } => {
                let gen_value = self.gen_eval(insert_value)?;
                match gen_value {
                    GenValue::Array { value: llvm_value, .. } => {
                        self.builder.build_call(
                            self.runtime.prepared_insert_exec_array.into(),
                            &[statement.into(), llvm_value.into()],
                            "insert_exec_array"
                        )?;
                    }
                    GenValue::Struct { .. } => {
                        let data_ptr = self.place_onto_stack(&gen_value)?;
                        self.builder.build_call(
                            self.runtime.prepared_insert_exec_row.into(),
                            &[statement.into(), data_ptr.into()],
                            "insert_exec_row"
                        )?;
                    }
                    _ => panic!("Unexpected insert value type")
                }
                Ok(GenValue::Void)
            },
            SemanticQuery::Update { assignments, where_clause, .. } => {
                for (i, assignment) in assignments.iter().enumerate() {
                    let gen_value = self.gen_eval(&assignment.value)?.as_llvm_basic_value();
                    let value_ptr = self.builder.build_alloca(
                        gen_value.get_type(),
                        &format!("update_assign_ptr_{}", i)
                    )?;
                    self.builder.build_store(value_ptr, gen_value)?;
                    self.builder.build_call(
                        self.runtime.prepared_update_bind_assignment,
                        &[
                            statement.into(),
                            self.context.i32_type().const_int(i as u64, false).into(),
                            self.get_qltype(&assignment.value.sem_type).into(),
                            value_ptr.into(),
                        ],
                        &format!("update_bind_assign_{}", i)
                    )?;
                }

                if let Some(WhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.place_onto_stack(&gen_value)?;
                    self.builder.build_call(
                        self.runtime.prepared_update_bind_where,
                        &[
                            statement.into(),
                            self.get_qltype(&value.sem_type).into(),
                            value_ptr.into(),
                        ],
                        "update_bind_where"
                    )?;
                }

                self.builder.build_call(
                    self.runtime.prepared_update_exec.into(),
                    &[statement.into()],
                    "execute_update"
                )?;
                Ok(GenValue::Void)
            },
            SemanticQuery::Delete { where_clause, .. } => {
                if let Some(WhereClause { value, .. }) = where_clause {
                    let gen_value = self.gen_eval(value)?;
                    let value_ptr = self.place_onto_stack(&gen_value)?;
                    self.builder.build_call(
                        self.runtime.prepared_delete_bind_where,
                        &[
                            statement.into(),
                            self.get_qltype(&value.sem_type).into(),
                            value_ptr.into(),
                        ],
                        "delete_bind_where"
                    )?;
                }

                self.builder.build_call(
                    self.runtime.prepared_delete_exec.into(),
                    &[statement.into()],
                    "execute_delete"
                )?.as_any_value_enum().into_pointer_value();

                Ok(GenValue::Void)
            }
        }
    }

    pub(super) fn finalize_query(
        &self,
        statement: PointerValue<'ctxt>,
        query: &SemanticQuery
    ) -> Result<(), CodeGenError> {
        match query {
            SemanticQuery::Select { .. } => {
                self.builder.build_call(
                    self.runtime.prepared_select_finalize.into(),
                    &[statement.into()],
                    "finalize_select"
                )?;
            },
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
        }
        Ok(())
    }
}