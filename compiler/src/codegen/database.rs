use inkwell::{AddressSpace, values::{AnyValue, FunctionValue, PointerValue}};

use crate::tokens::{DatasourceNode, QueryNode};

use super::{CodeGen, CodeGenError, QLValue, QLType};

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn gen_database_ptrs(&mut self, datasources: &[DatasourceNode]) {
        for datasource in datasources {
            let db_ptr_global = self.module
                .add_global(self.ptr_type(), Some(AddressSpace::default()), &datasource.name);
            db_ptr_global.set_initializer(&self.ptr_type().const_null());
            self.datasources.insert(datasource.name.clone(), db_ptr_global.as_pointer_value());
        }
    }

    pub(super) fn init_databases(
        &mut self,
        datasources: &[DatasourceNode],
        main_fn: FunctionValue<'ctxt>
    ) -> Result<PointerValue<'ctxt>, CodeGenError> {
        // Grab command line arguments
        let argc = main_fn.get_nth_param(0).unwrap().into_int_value();
        let argv = main_fn.get_nth_param(1).unwrap().into_pointer_value();

        // Create an array of global database pointers to feed to the runtime function
        let num_dbs = datasources.len() as u32;
        let db_ptr_arr_type = self.ptr_type().array_type(num_dbs);
        let db_ptr_arr = self.builder.build_alloca(db_ptr_arr_type, "db_ptr_arr")?;
        for (i, datasource) in datasources.iter().enumerate() {
            let db_ptr = *self.datasources.get(&datasource.name).unwrap();
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
        let num_dbs = self.datasources.len() as u32;
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

    pub fn gen_query(&self, query: &QueryNode) -> Result<QLValue<'ctxt>, CodeGenError> {
        let table = self.get_table(&query.table_name)?;

        let db_global_ptr = *self.datasources.get(&table.datasource_name).unwrap();
        let db_ptr = self.builder.build_load(self.ptr_type(), db_global_ptr, "db_ptr")?.into_pointer_value();
        
        let table_name_str = table.name_str.as_pointer_value();
        let type_info_ptr = table.type_info.as_pointer_value();
        
        let query_plan = self.builder.build_call(
            self.runtime_functions.query_plan_new.into(),
            &[table_name_str.into(), type_info_ptr.into()],
            "query_plan"
        )?.as_any_value_enum().into_pointer_value();
        
        if let Some(where_clause) = &query.where_clause {
            let column_index = table.get_column_index(&where_clause.column_name)?;
            let column_type = &table.fields[column_index as usize].ql_type;
            let column_name_str = table.column_name_strs[column_index as usize].as_pointer_value();
            
            // Evaluate the where clause value
            let where_value = where_clause.value.gen_eval(self)?;
            if where_value.get_type() != *column_type {
                return Err(CodeGenError::UnexpectedTypeError);
            }
            
            // Determine the QueryDataType enum value
            let query_data_type = match column_type {
                QLType::Integer => 0, // QUERY_INTEGER
                QLType::String => 1,  // QUERY_STRING
                _ => return Err(CodeGenError::UnexpectedTypeError),
            };
            let query_data_type_val = self.context.i32_type().const_int(query_data_type, false);
            
            // Allocate space for the value and store it
            let value_ptr = match where_value {
                QLValue::Integer(int_val) => {
                    let ptr = self.builder.build_alloca(self.int_type(), "where_value")?;
                    self.builder.build_store(ptr, int_val)?;
                    ptr
                },
                QLValue::String(str_ptr, _) => {
                    let ptr = self.builder.build_alloca(self.ptr_type(), "where_value")?;
                    self.builder.build_store(ptr, str_ptr)?;
                    ptr
                },
                _ => return Err(CodeGenError::UnexpectedTypeError),
            };
            
            // Call __ql__QueryPlan_set_where
            self.builder.build_call(
                self.runtime_functions.query_plan_set_where.into(),
                &[
                    query_plan.into(),
                    column_name_str.into(),
                    query_data_type_val.into(),
                    value_ptr.into(),
                ],
                "set_where"
            )?;
        }
        
        // Call __ql__QueryPlan_execute to execute the query
        let result_array = self.builder.build_call(
            self.runtime_functions.query_plan_execute.into(),
            &[db_ptr.into(), query_plan.into()],
            "query_result"
        )?.as_any_value_enum().into_pointer_value();
        
        // Return the result as a QLValue::Array
        Ok(QLValue::Array(
            result_array,
            QLType::Table(query.table_name.clone()),
            false
        ))
    }
}