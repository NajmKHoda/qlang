use std::{collections::HashSet};

use inkwell::{AddressSpace, basic_block::BasicBlock, builder::BuilderError, types::{BasicTypeEnum, StructType}, values::{BasicValueEnum, GlobalValue, IntValue}};

use crate::{codegen::QLFunction, tokens::{ColumnValueNode, TypedQNameNode}};

use super::{CodeGen, CodeGenError, QLType, QLValue};

pub(super) struct QLTableColumn {
    pub(super) name: String,
    pub(super) ql_type: QLType,
}

impl From<&TypedQNameNode> for QLTableColumn {
    fn from(node: &TypedQNameNode) -> Self {
        QLTableColumn {
            name: node.name.clone(),
            ql_type: node.ql_type.clone(),
        }
    }
}
    
pub(super) struct QLTable<'a> {
    pub(super) name: String,
    pub(super) datasource_name: String,
    pub(super) fields: Vec<QLTableColumn>,

    pub(super) struct_type: StructType<'a>,
    pub(super) type_info: GlobalValue<'a>,
    pub(super) name_str: GlobalValue<'a>,
    pub(super) column_name_strs: Vec<GlobalValue<'a>>,

    pub(super) copy_fn: Option<QLFunction<'a>>,
    pub(super) drop_fn: Option<QLFunction<'a>>,
}

impl<'a> QLTable<'a> {
    pub fn get_column_index(&self, column_name: &str) -> Result<u32, CodeGenError> {
        self.fields.iter().position(|c| c.name == column_name)
            .map(|idx| idx as u32)
            .ok_or_else(|| CodeGenError::UndefinedTableColumnError(column_name.to_string(), self.name.clone()))
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn get_table(&self, name: &str) -> Result<&QLTable<'ctxt>, CodeGenError> {
        self.tables.get(name)
            .ok_or_else(|| CodeGenError::UndefinedTableError(name.to_string()))
    }

    pub fn gen_table(&mut self, name: &str, datasource_name: &str, fields: &[TypedQNameNode]) -> Result<(), CodeGenError> {
        if !self.datasources.contains_key(datasource_name) {
            return Err(CodeGenError::UndefinedDatasourceError(datasource_name.to_string()));
        }

        let field_types = fields.iter()
            .map(|f| self.try_get_nonvoid_type(&f.ql_type))
            .collect::<Result<Vec<BasicTypeEnum>, CodeGenError>>()?;
        
        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&field_types, false);

        let table_fields: Vec<QLTableColumn> = fields.iter().map(|f| f.into()).collect();
        
        // Check if any fields are heap-allocated
        let has_heap_fields = table_fields.iter().any(|f| !f.ql_type.is_primitive());
        
        let (copy_fn, drop_fn) = if has_heap_fields {
            // Create copy function
            let copy_fn_type = self.context.void_type().fn_type(&[struct_type.into()], false);
            let copy_fn_value = self.module.add_function(
                &format!("__ql__{}_copy", name),
                copy_fn_type,
                None
            );
            let copy_entry = self.context.append_basic_block(copy_fn_value, "entry");
            self.builder.position_at_end(copy_entry);
            
            let struct_arg = copy_fn_value.get_nth_param(0).unwrap().into_struct_value();
            
            for (i, field) in table_fields.iter().enumerate() {
                if !field.ql_type.is_primitive() {
                    let field_value = self.builder.build_extract_value(
                        struct_arg,
                        i as u32,
                        &format!("{}.{}", name, field.name)
                    )?;
                    let ql_value = field.ql_type.to_value(field_value, true);
                    self.add_ref(&ql_value)?;
                }
            }
            
            self.builder.build_return(None)?;
            
            // Create drop function
            let drop_fn_type = self.context.void_type().fn_type(&[struct_type.into()], false);
            let drop_fn_value = self.module.add_function(
                &format!("__ql__{}_drop", name),
                drop_fn_type,
                None
            );
            let drop_entry = self.context.append_basic_block(drop_fn_value, "entry");
            self.builder.position_at_end(drop_entry);
            
            let struct_arg = drop_fn_value.get_nth_param(0).unwrap().into_struct_value();
            
            for (i, field) in table_fields.iter().enumerate() {
                if !field.ql_type.is_primitive() {
                    let field_llvm_value = self.builder.build_extract_value(
                        struct_arg,
                        i as u32,
                        &format!("{}.{}", name, field.name)
                    )?;
                    let field_value = field.ql_type.to_value(field_llvm_value, true);
                    self.remove_ref(field_value)?;
                }
            }
            
            self.builder.build_return(None)?;
            
            (Some(QLFunction {
                name: format!("__ql__{}_copy", name),
                llvm_function: copy_fn_value,
                return_type: QLType::Void,
                params: vec![],
            }), Some(QLFunction {
                name: format!("__ql__{}_drop", name),
                llvm_function: drop_fn_value,
                return_type: QLType::Void,
                params: vec![],
            }))
        } else {
            (None, None)
        };
        
        // Create elem_drop wrapper if drop_fn exists
        let elem_drop_fn_ptr = if let Some(ref drop_fn_ref) = drop_fn {
            let elem_drop_type = self.context.void_type().fn_type(
                &[self.context.ptr_type(Default::default()).into()],
                false
            );
            let elem_drop_fn = self.module.add_function(
                &format!("__ql__{}_elem_drop", name),
                elem_drop_type,
                None
            );
            let elem_drop_entry = self.context.append_basic_block(elem_drop_fn, "entry");
            self.builder.position_at_end(elem_drop_entry);
            
            let ptr_arg = elem_drop_fn.get_nth_param(0).unwrap().into_pointer_value();
            let struct_val = self.builder.build_load(
                struct_type,
                ptr_arg,
                "struct_val"
            )?.into_struct_value();
            
            self.builder.build_call(
                drop_fn_ref.llvm_function,
                &[struct_val.into()],
                "call_drop"
            )?;
            
            self.builder.build_return(None)?;
            
            Some(elem_drop_fn.as_global_value().as_pointer_value())
        } else {
            None
        };
        
        let set_nth_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(Default::default()).into(),
                self.context.i32_type().into(),
                self.context.ptr_type(Default::default()).into()
            ],
            false
        );
        let set_nth_fn = self.module.add_function(
            &format!("__ql__{}_set_nth", name),
            set_nth_type,
            None
        );

        // Construct the set-nth function
        let set_nth_entry = self.context.append_basic_block(set_nth_fn, "entry");
        self.builder.position_at_end(set_nth_entry);

        let struct_ptr = set_nth_fn.get_nth_param(0).unwrap().into_pointer_value();
        let struct_index = set_nth_fn.get_nth_param(1).unwrap().into_int_value();
        let value_ptr = set_nth_fn.get_nth_param(2).unwrap().into_pointer_value();
        
        let cases = (0..table_fields.len())
            .into_iter()
            .map(|i| -> Result<(IntValue, BasicBlock), CodeGenError> {
                let case_value = self.context.i32_type().const_int(i as u64, false);
                let case_block = self.context.append_basic_block(set_nth_fn, &format!("case_{}", i));
                self.builder.position_at_end(case_block);

                let field_ptr = self.builder.build_struct_gep(
                    struct_type,
                    struct_ptr,
                    i as u32,
                    &format!("{}.{}", name, table_fields[i].name)
                )?;

                let loaded_value = self.builder.build_load(
                    self.try_get_nonvoid_type(&table_fields[i].ql_type)?,
                    value_ptr,
                    "loaded_value"
                )?;

                self.builder.build_store(field_ptr, loaded_value)?;
                self.builder.build_return(None)?;
                Ok((case_value, case_block))
            })
            .collect::<Result<Vec<(IntValue, BasicBlock)>, CodeGenError>>()?;
        
        let else_block = self.context.append_basic_block(set_nth_fn, "else");
        self.builder.position_at_end(else_block);
        self.builder.build_return(None)?;

        self.builder.position_at_end(set_nth_entry);
        self.builder.build_switch(struct_index, else_block, &cases)?;

        // Create and initialize type_info global
        let type_info = self.module.add_global(
            self.runtime_functions.type_info_type,
            Some(AddressSpace::default()),
            &format!("__ql__{}_type_info", name)
        );
        
        // Initialize with struct size and elem_drop function pointer
        let size_value = struct_type.size_of().unwrap();
        let elem_drop_value = if let Some(fn_ptr) = elem_drop_fn_ptr {
            fn_ptr
        } else {
            self.context.ptr_type(Default::default()).const_null()
        };
        
        let type_info_init = self.runtime_functions.type_info_type.const_named_struct(&[
            size_value.into(),
            elem_drop_value.into(),
            set_nth_fn.as_global_value().as_pointer_value().into(),
        ]);
        type_info.set_initializer(&type_info_init);

        let name_str = self.builder.build_global_string_ptr(name, &format!("{}_name", name))?;
        let column_name_strs = table_fields.iter().enumerate().map(|(i, col)| {
            self.builder.build_global_string_ptr(&col.name, &format!("{}_col_{}", name, i))
        }).collect::<Result<Vec<GlobalValue>, BuilderError>>()?;
        
        let table = QLTable {
            name: name.to_string(),
            datasource_name: datasource_name.to_string(),
            name_str,
            column_name_strs,
            struct_type,
            fields: table_fields,
            type_info,
            copy_fn,
            drop_fn,
        };

        self.tables.insert(name.to_string(), table);

        Ok(())
    }

    pub fn gen_table_row(&self, table_name: &str, columns: &[ColumnValueNode]) -> Result<QLValue<'ctxt>, CodeGenError> {
        let table = self.get_table(table_name)?;
        let row_ptr = self.builder.build_alloca(table.struct_type, &format!("{}.row.store", table_name))?;
        let mut remaining_columns: HashSet<_> = (0..table.fields.len() as u32).collect();
        for column in columns {
            let column_index = table.get_column_index(&column.name)?;
            if !remaining_columns.contains(&column_index) {
                return Err(CodeGenError::DuplicateColumnAssignmentError(column.name.clone(), table_name.to_string()));
            }

            let column_ptr = self.builder.build_struct_gep(
                table.struct_type,
                row_ptr, 
                column_index,
                &format!("{}.{}", table_name, column.name)
            )?;

            let column_value = column.value.gen_eval(&self)?;
            if column_value.get_type() != table.fields[column_index as usize].ql_type {
                return Err(CodeGenError::UnexpectedTypeError);
            }

            self.add_ref(&column_value)?;
            self.builder.build_store(column_ptr, BasicValueEnum::try_from(column_value)?)?;
            remaining_columns.remove(&column_index);
        }

        if remaining_columns.len() != 0 {
            return Err(CodeGenError::MissingColumnAssignmentError(table_name.to_string()));
        }

        let struct_val = self.builder.build_load(
            table.struct_type,
            row_ptr,
            &format!("{}.row.load", table_name)
        )?.into_struct_value();
        Ok(QLValue::TableRow(struct_val, table_name.to_string(), false))
    }

    pub fn get_column_value(&self, table_row: QLValue<'ctxt>, column_name: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let QLValue::TableRow(struct_val, table_name, _) = table_row {
            let table = self.get_table(&table_name)?;
            let column_index = table.get_column_index(column_name)?;
            let column_type = &table.fields[column_index as usize].ql_type;
            let loaded_val = self.builder.build_extract_value(
                struct_val,
                column_index,
                &format!("{}.{}", table_name, column_name)
            )?;

            Ok(column_type.to_value(loaded_val, true))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}