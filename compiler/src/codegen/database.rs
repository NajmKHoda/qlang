use inkwell::{AddressSpace, values::{AnyValue, BasicValue, BasicValueEnum, FunctionValue, PointerValue}};

use crate::{codegen::{data::GenValue}, semantics::{Ownership, SemanticDatasource, SemanticExpression, SemanticType, SemanticTypeKind, WhereClause}};

use super::{CodeGen, CodeGenError};

impl<'ctxt> CodeGen<'ctxt> {
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
            self.runtime_functions.init_dbs.into(),
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
            self.runtime_functions.close_dbs.into(),
            &[
                self.context.i32_type().const_int(num_dbs as u64, false).into(),
                db_ptr_arr.into(),
            ],
            "close_dbs_call"
        )?;
        Ok(())
    }

    pub fn gen_select_query(&mut self, table_id: u32, where_clause_opt: &Option<WhereClause>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let where_value = match where_clause_opt {
            Some(where_clause) => self.gen_eval(&where_clause.value)?,
            None => GenValue::Void,
        };

        let table = &self.program.tables[&table_id];
        let assoc_struct = &self.program.structs[&table.struct_id];
        let table_info = &self.table_info[&table_id];
        let db_global_ptr = self.datasource_ptrs[&table.datasource_id];
        let db_ptr = self.builder.build_load(self.ptr_type(), db_global_ptr, "db_ptr")?.into_pointer_value();
        
        let table_name_str = table_info.name_str.as_pointer_value();
        let type_info_ptr = self.struct_info[&table.struct_id].type_info.as_pointer_value();
        
        let query_plan = self.builder.build_call(
            self.runtime_functions.select_query_plan_new.into(),
            &[
                table_name_str.into(),
                type_info_ptr.into(),
                self.context.i32_type().const_zero().into(),
            ],
            "query_plan"
        )?.as_any_value_enum().into_pointer_value();
        
        if let Some(WhereClause { column_name, .. }) = where_clause_opt {
            let column_index = assoc_struct.field_order.iter()
                .position(|name| name == column_name).unwrap();
            let column_type = &assoc_struct.fields[column_name];
            let column_name_str = table_info.column_name_strs[column_index].as_pointer_value();
            
            let query_data_type = match column_type.kind() {
                SemanticTypeKind::Integer => 0, // QUERY_DATA_INTEGER
                SemanticTypeKind::String => 1,  // QUERY_DATA_STRING
                _ => panic!("Unexpected type in WHERE clause"),
            };
            let query_data_type_val = self.context.i32_type().const_int(query_data_type, false);
            
            // Allocate space for the value and store it
            let value_ptr = match where_value {
                GenValue::Integer(int_val) => {
                    let ptr = self.builder.build_alloca(self.int_type(), "where_value")?;
                    self.builder.build_store(ptr, int_val)?;
                    ptr
                },
                GenValue::String { value: str_ptr, .. } => {
                    let ptr = self.builder.build_alloca(self.ptr_type(), "where_value")?;
                    self.builder.build_store(ptr, str_ptr)?;
                    ptr
                },
                _ => panic!("Unexpected query value"),
            };
            
            // Call __ql__SelectQueryPlan_set_where
            self.builder.build_call(
                self.runtime_functions.select_query_plan_set_where.into(),
                &[
                    query_plan.into(),
                    column_name_str.into(),
                    query_data_type_val.into(),
                    value_ptr.into(),
                ],
                "set_where"
            )?;
        }
        
        // Prepare the query
        let prepared_query = self.builder.build_call(
            self.runtime_functions.select_query_plan_prepare.into(),
            &[db_ptr.into(), query_plan.into()],
            "prepared_query"
        )?.as_any_value_enum().into_pointer_value();
        
        // Execute the prepared query
        let result_array = self.builder.build_call(
            self.runtime_functions.prepared_query_execute.into(),
            &[prepared_query.into()],
            "query_result"
        )?.as_any_value_enum().into_pointer_value();
        
        // Free the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_remove_ref.into(),
            &[prepared_query.into()],
            "free_prepared_query"
        )?;
        
        // Return the result as a GenValue::Array
        Ok(GenValue::Array {
            value: result_array,
            elem_type: SemanticType::new(SemanticTypeKind::NamedStruct(
                assoc_struct.id,
                assoc_struct.name.clone()
            )),
            ownership: Ownership::Owned,
        })
    }

    pub fn gen_insert_query(&mut self, table_id: u32, value: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let insert_data = self.gen_eval(value)?;
        let table = &self.program.tables[&table_id];
        let table_info = &self.table_info[&table_id];
        let db_global_ptr = self.datasource_ptrs[&table.datasource_id];
        let db_ptr = self.builder.build_load(self.ptr_type(), db_global_ptr, "db_ptr")?.into_pointer_value();
        
        let table_name_str = table_info.name_str.as_pointer_value();
        let type_info_ptr = self.struct_info[&table_id].type_info.as_pointer_value();
        
        let llvm_insert_data: BasicValueEnum = match &insert_data {
            GenValue::Struct { value: struct_val, .. } => struct_val.as_basic_value_enum(),
            _ => panic!()
        };
        let insert_data_ptr = self.builder.build_alloca(llvm_insert_data.get_type(), "insert_data")?;
        self.builder.build_store(insert_data_ptr, llvm_insert_data)?;
        
        // Create the insert query plan
        let query_plan_ptr = self.builder.build_call(
            self.runtime_functions.insert_query_plan_new.into(),
            &[
                table_name_str.into(),
                type_info_ptr.into(),
                self.context.i32_type().const_zero().into(),
                self.bool_type().const_int(false as u64, false).into(),
                insert_data_ptr.into(),
            ],
            "insert_query_plan"
        )?.as_any_value_enum().into_pointer_value();

        // Prepare the query
        let prepared_query = self.builder.build_call(
            self.runtime_functions.insert_query_plan_prepare.into(),
            &[db_ptr.into(), query_plan_ptr.into()],
            "prepared_query"
        )?.as_any_value_enum().into_pointer_value();

        // Execute the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_execute.into(),
            &[prepared_query.into()],
            "execute_insert"
        )?;
        
        // Free the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_remove_ref.into(),
            &[prepared_query.into()],
            "free_prepared_query"
        )?;
        
        self.remove_if_owned(insert_data)?;

        Ok(GenValue::Void)
    }

    pub fn gen_delete_query(&mut self, table_id: u32, where_clause_opt: &Option<WhereClause>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let where_value = match where_clause_opt {
            Some(where_clause) => self.gen_eval(&where_clause.value)?,
            None => GenValue::Void,
        };

        let table = &self.program.tables[&table_id];
        let assoc_struct = &self.program.structs[&table.struct_id];
        let table_info = &self.table_info[&table_id];
        let db_global_ptr = self.datasource_ptrs[&table.datasource_id];
        let db_ptr = self.builder.build_load(self.ptr_type(), db_global_ptr, "db_ptr")?.into_pointer_value();
        
        let table_name_str = table_info.name_str.as_pointer_value();
        
        let query_plan = self.builder.build_call(
            self.runtime_functions.delete_query_plan_new.into(),
            &[
                table_name_str.into(),
                self.context.i32_type().const_zero().into(),
            ],
            "delete_query_plan"
        )?.as_any_value_enum().into_pointer_value();
        
        if let Some(WhereClause { column_name, .. }) = where_clause_opt {
            let column_index = assoc_struct.field_order.iter()
                .position(|name| name == column_name).unwrap();
            let column_type = &assoc_struct.fields[column_name];
            let column_name_str = table_info.column_name_strs[column_index].as_pointer_value();
            
            let query_data_type = match column_type.kind() {
                SemanticTypeKind::Integer => 0, // QUERY_DATA_INTEGER
                SemanticTypeKind::String => 1,  // QUERY_DATA_STRING
                _ => panic!("Unexpected query value"),
            };
            let query_data_type_val = self.context.i32_type().const_int(query_data_type, false);
            
            // Allocate space for the value and store it
            let value_ptr = match where_value {
                GenValue::Integer(int_val) => {
                    let ptr = self.builder.build_alloca(self.int_type(), "where_value")?;
                    self.builder.build_store(ptr, int_val)?;
                    ptr
                },
                GenValue::String { value: str_ptr, .. } => {
                    let ptr = self.builder.build_alloca(self.ptr_type(), "where_value")?;
                    self.builder.build_store(ptr, str_ptr)?;
                    ptr
                },
                _ => panic!("Unexpected query value"),
            };
            
            // Call __ql__DeleteQueryPlan_set_where
            self.builder.build_call(
                self.runtime_functions.delete_query_plan_set_where.into(),
                &[
                    query_plan.into(),
                    column_name_str.into(),
                    query_data_type_val.into(),
                    value_ptr.into(),
                ],
                "set_where"
            )?;
        }
        
        // Prepare the query
        let prepared_query = self.builder.build_call(
            self.runtime_functions.delete_query_plan_prepare.into(),
            &[db_ptr.into(), query_plan.into()],
            "prepared_query"
        )?.as_any_value_enum().into_pointer_value();
        
        // Execute the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_execute.into(),
            &[prepared_query.into()],
            "execute_delete"
        )?;
        
        // Free the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_remove_ref.into(),
            &[prepared_query.into()],
            "free_prepared_query"
        )?;

        Ok(GenValue::Void)
    }

    pub fn gen_update_query(&mut self, table_id: u32, assignments: &[(String, SemanticExpression)], where_clause_opt: &Option<WhereClause>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let assignment_values = assignments.iter()
            .map(|(col_name, expr)| {
                match self.gen_eval(expr) {
                    Ok(val) => Ok((col_name.as_str(), val)),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<(&str, GenValue)>, _>>()?;

        let where_value = match where_clause_opt {
            Some(where_clause) => self.gen_eval(&where_clause.value)?,
            None => GenValue::Void,
        };

        let table = &self.program.tables[&table_id];
        let assoc_struct = &self.program.structs[&table.struct_id];
        let table_info = &self.table_info[&table_id];
        let db_global_ptr = self.datasource_ptrs[&table.datasource_id];
        let db_ptr = self.builder.build_load(self.ptr_type(), db_global_ptr, "db_ptr")?.into_pointer_value();
        
        let table_name_str = table_info.name_str.as_pointer_value();
        let type_info_ptr = self.struct_info[&table.struct_id].type_info.as_pointer_value();
        
        let query_plan = self.builder.build_call(
            self.runtime_functions.update_query_plan_new.into(),
            &[
                table_name_str.into(),
                type_info_ptr.into(),
                self.context.i32_type().const_zero().into(),
            ],
            "update_query_plan"
        )?.as_any_value_enum().into_pointer_value();

        // Add all assignments
        for (column_name, assignment_value) in assignment_values {
            let column_index = assoc_struct.field_order.iter()
                .position(|name| name == column_name).unwrap();
            let column_type = &assoc_struct.fields[column_name];
            let column_name_str = table_info.column_name_strs[column_index].as_pointer_value();
            
            let query_data_type = match column_type.kind() {
                SemanticTypeKind::Integer => 0, // QUERY_DATA_INTEGER
                SemanticTypeKind::String => 1,  // QUERY_DATA_STRING
                _ => panic!("Unexpected query value"),
            };
            let query_data_type_val = self.context.i32_type().const_int(query_data_type, false);
            
            // Allocate space for the value and store it
            let value_ptr = match assignment_value {
                GenValue::Integer(int_val) => {
                    let ptr = self.builder.build_alloca(self.int_type(), "assignment_value")?;
                    self.builder.build_store(ptr, int_val)?;
                    ptr
                },
                GenValue::String { value: str_ptr, .. } => {
                    let ptr = self.builder.build_alloca(self.ptr_type(), "assignment_value")?;
                    self.builder.build_store(ptr, str_ptr)?;
                    ptr
                },
                _ => panic!("Unexpected query value"),
            };
            
            // Call __ql__UpdateQueryPlan_add_assignment
            self.builder.build_call(
                self.runtime_functions.update_query_plan_add_assignment.into(),
                &[
                    query_plan.into(),
                    column_name_str.into(),
                    query_data_type_val.into(),
                    value_ptr.into(),
                ],
                "add_assignment"
            )?;
        }
        
        // Handle WHERE clause if present
        if let Some(WhereClause { column_name, .. }) = where_clause_opt {
            let column_index = assoc_struct.field_order.iter()
                .position(|name| name == column_name).unwrap();
            let column_type = &assoc_struct.fields[column_name];
            let column_name_str = table_info.column_name_strs[column_index].as_pointer_value();
            
            let query_data_type = match column_type.kind() {
                SemanticTypeKind::Integer => 0, // QUERY_DATA_INTEGER
                SemanticTypeKind::String => 1,  // QUERY_DATA_STRING
                _ => panic!("Unexpected query value"),
            };
            let query_data_type_val = self.context.i32_type().const_int(query_data_type, false);
            
            // Allocate space for the value and store it
            let value_ptr = match where_value {
                GenValue::Integer(int_val) => {
                    let ptr = self.builder.build_alloca(self.int_type(), "where_value")?;
                    self.builder.build_store(ptr, int_val)?;
                    ptr
                },
                GenValue::String { value: str_ptr, .. } => {
                    let ptr = self.builder.build_alloca(self.ptr_type(), "where_value")?;
                    self.builder.build_store(ptr, str_ptr)?;
                    ptr
                },
                _ => panic!("Unexpected query value"),
            };
            
            // Call __ql__UpdateQueryPlan_set_where
            self.builder.build_call(
                self.runtime_functions.update_query_plan_set_where.into(),
                &[
                    query_plan.into(),
                    column_name_str.into(),
                    query_data_type_val.into(),
                    value_ptr.into(),
                ],
                "set_where"
            )?;
        }
        
        // Prepare the query
        let prepared_query = self.builder.build_call(
            self.runtime_functions.update_query_plan_prepare.into(),
            &[db_ptr.into(), query_plan.into()],
            "prepared_query"
        )?.as_any_value_enum().into_pointer_value();
        
        // Execute the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_execute.into(),
            &[prepared_query.into()],
            "execute_update"
        )?;
        
        // Free the prepared query
        self.builder.build_call(
            self.runtime_functions.prepared_query_remove_ref.into(),
            &[prepared_query.into()],
            "free_prepared_query"
        )?;

        Ok(GenValue::Void)
    }
}