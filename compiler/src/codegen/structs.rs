use std::collections::HashMap;

use inkwell::{AddressSpace, basic_block::BasicBlock, types::{BasicTypeEnum, StructType}, values::{FunctionValue, GlobalValue, IntValue}};

use crate::{codegen::{CodeGenError, data::GenValue}, semantics::{Ownership, SemanticExpression, SemanticStruct, SemanticTypeKind}};

use super::CodeGen;

pub(super) struct GenStructInfo<'a> {
    pub(super) struct_type: StructType<'a>,
    pub(super) type_info: GlobalValue<'a>,
    pub(super) copy_fn: Option<FunctionValue<'a>>,
    pub(super) drop_fn: Option<FunctionValue<'a>>,
}

impl<'ctxt> CodeGen<'ctxt> {
    fn create_copy_fn(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let copy_fn_type = self.void_type().fn_type(&[struct_type.into()], false);
        let copy_fn_value = self.module.add_function(
            &format!("__ql__{}_copy", sem_struct.name),
            copy_fn_type,
            None
        );
        let copy_entry = self.context.append_basic_block(copy_fn_value, "entry");
        self.builder.position_at_end(copy_entry);
        
        let struct_arg = copy_fn_value.get_nth_param(0).unwrap().into_struct_value();
        
        for (i, field_name) in sem_struct.field_order.iter().enumerate() {
            let field_type = &sem_struct.fields[field_name];
            if field_type.can_be_owned() {
                let field_value = self.builder.build_extract_value(
                    struct_arg,
                    i as u32,
                    &format!("{}.{}", sem_struct.name, field_name)
                )?;
                let ql_value = GenValue::new(field_type, field_value, Ownership::Borrowed);
                self.add_ref(&ql_value)?;
            }
        }
        
        self.builder.build_return(None)?;
        Ok(copy_fn_value)
    }

    fn create_drop_fn(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let drop_fn_type = self.void_type().fn_type(&[struct_type.into()], false);
        let drop_fn_value = self.module.add_function(
            &format!("__ql__{}_drop", sem_struct.name),
            drop_fn_type,
            None
        );
        let drop_entry = self.context.append_basic_block(drop_fn_value, "entry");
        self.builder.position_at_end(drop_entry);
        
        let struct_arg = drop_fn_value.get_nth_param(0).unwrap().into_struct_value();
        
        for (i, field_name) in sem_struct.field_order.iter().enumerate() {
            let field_type = &sem_struct.fields[field_name];
            if field_type.can_be_owned() {
                let field_llvm_value = self.builder.build_extract_value(
                    struct_arg,
                    i as u32,
                    &format!("{}.{}", sem_struct.name, field_name)
                )?;
                let field_value = GenValue::new(field_type, field_llvm_value, Ownership::Borrowed);
                self.remove_ref(field_value)?;
            }
        }

        self.builder.build_return(None)?;
        Ok(drop_fn_value)
    }

    fn create_elem_drop_fn(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>,
        drop_fn: FunctionValue<'ctxt>,
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let elem_drop_type = self.void_type().fn_type(
            &[self.context.ptr_type(Default::default()).into()],
            false
        );
        let elem_drop_fn = self.module.add_function(
            &format!("__ql__{}_elem_drop", sem_struct.name),
            elem_drop_type,
            None
        );
        let elem_drop_entry = self.context.append_basic_block(elem_drop_fn, "entry");
        self.builder.position_at_end(elem_drop_entry);
        
        let ptr_arg = elem_drop_fn.get_nth_param(0).unwrap().into_pointer_value();
        let struct_val = self.builder.build_load(struct_type, ptr_arg, "struct_val")?
        .into_struct_value();
        
        self.builder.build_call(drop_fn, &[struct_val.into()], "call_drop")?;
        
        self.builder.build_return(None)?;
        Ok(elem_drop_fn)
    }

    fn create_set_nth_fn(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let set_nth_type = self.void_type().fn_type(
            &[
                self.context.ptr_type(Default::default()).into(),
                self.context.i32_type().into(),
                self.context.ptr_type(Default::default()).into()
            ],
            false
        );
        let set_nth_fn = self.module.add_function(
            &format!("__ql__{}_set_nth", sem_struct.name),
            set_nth_type,
            None
        );

        // Construct the set-nth function
        let set_nth_entry = self.context.append_basic_block(set_nth_fn, "entry");
        self.builder.position_at_end(set_nth_entry);

        let struct_ptr = set_nth_fn.get_nth_param(0).unwrap().into_pointer_value();
        let struct_index = set_nth_fn.get_nth_param(1).unwrap().into_int_value();
        let value_ptr = set_nth_fn.get_nth_param(2).unwrap().into_pointer_value();
        
        let cases = sem_struct.field_order
            .iter().enumerate()
            .map(|(i, field_name)| -> Result<(IntValue, BasicBlock), CodeGenError> {
                let case_value = self.context.i32_type().const_int(i as u64, false);
                let case_block = self.context.append_basic_block(set_nth_fn, &format!("case_{}", i));
                self.builder.position_at_end(case_block);

                let field_ptr = self.builder.build_struct_gep(
                    struct_type,
                    struct_ptr,
                    i as u32,
                    &format!("{}.{}", sem_struct.name, field_name)
                )?;

                let loaded_value = self.builder.build_load(
                    self.llvm_basic_type(&sem_struct.fields[field_name]),
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
        Ok(set_nth_fn)
    }

    fn create_get_nth_fn(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let get_nth_type = self.void_type().fn_type(
            &[
                self.context.ptr_type(Default::default()).into(),
                self.context.i32_type().into(),
                self.context.ptr_type(Default::default()).into(),
                self.context.ptr_type(Default::default()).into()
            ],
            false
        );
        let get_nth_fn = self.module.add_function(
            &format!("__ql__{}_get_nth", sem_struct.name),
            get_nth_type,
            None
        );

        // Construct the get-nth function
        let get_nth_entry = self.context.append_basic_block(get_nth_fn, "entry");
        self.builder.position_at_end(get_nth_entry);

        let get_struct_ptr = get_nth_fn.get_nth_param(0).unwrap().into_pointer_value();
        let get_struct_index = get_nth_fn.get_nth_param(1).unwrap().into_int_value();
        let datatype_ptr = get_nth_fn.get_nth_param(2).unwrap().into_pointer_value();
        let out_value_ptr = get_nth_fn.get_nth_param(3).unwrap().into_pointer_value();
        
        let get_cases = sem_struct.field_order
            .iter().enumerate()
            .map(|(i, field_name)| -> Result<(IntValue, BasicBlock), CodeGenError> {
                let case_value = self.context.i32_type().const_int(i as u64, false);
                let case_block = self.context.append_basic_block(get_nth_fn, &format!("get_case_{}", i));
                self.builder.position_at_end(case_block);

                let field_ptr = self.builder.build_struct_gep(
                    struct_type,
                    get_struct_ptr,
                    i as u32,
                    &format!("{}.{}", sem_struct.name, field_name)
                )?;

                // Determine QueryDataType based on QLType
                let datatype_value = match sem_struct.fields[field_name].kind() {
                    SemanticTypeKind::Integer => self.context.i32_type().const_int(0, false), // QUERY_DATA_INTEGER = 0
                    SemanticTypeKind::String => self.context.i32_type().const_int(1, false),  // QUERY_DATA_STRING = 1
                    _ => return Err(CodeGenError::UnexpectedTypeError),
                };

                // Store the datatype
                self.builder.build_store(datatype_ptr, datatype_value)?;

                // Store the field pointer into out_value_ptr
                let field_ptr_as_opaque = self.builder.build_pointer_cast(
                    field_ptr,
                    self.context.ptr_type(Default::default()),
                    "field_ptr_cast"
                )?;
                self.builder.build_store(out_value_ptr, field_ptr_as_opaque)?;
                
                self.builder.build_return(None)?;
                Ok((case_value, case_block))
            })
            .collect::<Result<Vec<(IntValue, BasicBlock)>, CodeGenError>>()?;
        
        let get_else_block = self.context.append_basic_block(get_nth_fn, "get_else");
        self.builder.position_at_end(get_else_block);
        self.builder.build_return(None)?;

        self.builder.position_at_end(get_nth_entry);
        self.builder.build_switch(get_struct_index, get_else_block, &get_cases)?;

        Ok(get_nth_fn)
    }

    fn create_type_info(
        &self,
        sem_struct: &SemanticStruct,
        struct_type: StructType<'ctxt>,
        elem_drop_fn: Option<FunctionValue<'ctxt>>,
        set_nth_fn: FunctionValue<'ctxt>,
        get_nth_fn: FunctionValue<'ctxt>
    ) -> Result<GlobalValue<'ctxt>, CodeGenError> {
        let type_info = self.module.add_global(
            self.runtime_functions.type_info_type,
            Some(AddressSpace::default()),
            &format!("__ql__{}_type_info", sem_struct.name)
        );

        let size_value = struct_type.size_of().unwrap();
        let elem_drop_value = if let Some(fn_ptr) = elem_drop_fn {
            fn_ptr.as_global_value().as_pointer_value()
        } else {
            self.context.ptr_type(Default::default()).const_null()
        };
        let num_columns_value = self.context.i32_type().const_int(sem_struct.field_order.len() as u64, false);
        
        let type_info_init = self.runtime_functions.type_info_type.const_named_struct(&[
            size_value.into(),
            elem_drop_value.into(),
            num_columns_value.into(),
            set_nth_fn.as_global_value().as_pointer_value().into(),
            get_nth_fn.as_global_value().as_pointer_value().into(),
        ]);
        type_info.set_initializer(&type_info_init);

        Ok(type_info)
    }

    pub fn gen_struct(&mut self, sem_struct: &SemanticStruct) -> Result<(), CodeGenError> {
        let field_types = sem_struct.field_order.iter()
            .map(|field_name| {
                let field_type = &sem_struct.fields[field_name];
                self.llvm_basic_type(field_type)
            })
            .collect::<Vec<BasicTypeEnum>>();

        let struct_type = self.context.opaque_struct_type(&sem_struct.name);
        struct_type.set_body(&field_types, false);

        let has_heap_fields = sem_struct.fields
            .values()
            .any(|field_type| field_type.can_be_owned());

        let (copy_fn, drop_fn) = if has_heap_fields {
            let copy_fn_value = self.create_copy_fn(sem_struct, struct_type)?;
            let drop_fn_value = self.create_drop_fn(sem_struct, struct_type)?;
            (Some(copy_fn_value), Some(drop_fn_value))
        } else {
            (None, None)
        };

        let elem_drop_fn = drop_fn.map(|df| {
            self.create_elem_drop_fn(sem_struct, struct_type, df)
        }).transpose()?;

        let get_nth_fn = self.create_get_nth_fn(sem_struct, struct_type)?;
        let set_nth_fn = self.create_set_nth_fn(sem_struct, struct_type)?;

        let type_info = self.create_type_info(
            sem_struct,
            struct_type,
            elem_drop_fn,
            set_nth_fn,
            get_nth_fn
        )?;

        self.struct_info.insert(sem_struct.id, GenStructInfo {
            struct_type,
            copy_fn,
            drop_fn,
            type_info,
        });

        Ok(())
    }

    pub fn gen_struct_value(
        &mut self,
        struct_id: u32,
        columns: &HashMap<String, SemanticExpression>
    ) -> Result<GenValue<'ctxt>, CodeGenError> {
        let sem_struct = &self.program.structs[&struct_id];
        let column_values = sem_struct.field_order.iter()
            .map(|col_name| self.gen_eval(&columns[col_name]))
            .collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;

        let struct_info = &self.struct_info[&struct_id];
        let struct_ptr = self.builder.build_alloca(struct_info.struct_type, &format!("{}_store", sem_struct.name))?;
        for (column_name, column_value) in sem_struct.field_order.iter().zip(column_values) {
            let column_index = sem_struct.field_order.iter()
                .position(|x| x == column_name).unwrap() as u32;
            let column_ptr = self.builder.build_struct_gep(
                struct_info.struct_type,
                struct_ptr, 
                column_index,
                &format!("{}.{}", sem_struct.name, column_name)
            )?;

            self.add_ref(&column_value)?;
            self.builder.build_store(column_ptr, column_value.as_llvm_basic_value())?;
        }

        let struct_val = self.builder.build_load(
            struct_info.struct_type,
            struct_ptr,
            &format!("{}_load", sem_struct.name)
        )?.into_struct_value();

        Ok(GenValue::Struct {
            value: struct_val,
            struct_id, 
            ownership: Ownership::Owned
        })
    }

    pub fn get_field_value(&self, struct_value: GenValue<'ctxt>, index: u32) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Struct { value: llvm_value, struct_id, .. } = struct_value else {
            panic!("Expected struct value");
        };
    
        let sem_struct = &self.program.structs[&struct_id];
        let field_name = &sem_struct.field_order[index as usize];
        let field_type = &sem_struct.fields[field_name];
        let loaded_val = self.builder.build_extract_value(
            llvm_value,
            index,
            &format!("{}.{}", sem_struct.name, field_name)
        )?;

        Ok(GenValue::new(field_type, loaded_val, Ownership::Borrowed))
    }
}