use std::{collections::HashMap};

use inkwell::{AddressSpace, types::{BasicTypeEnum, StructType}, values::{FunctionValue, GlobalValue}};

use crate::{codegen::{CodeGenError, data::GenValue, database::ColumnType}, semantics::{Ownership, SemanticExpression, SemanticStruct, SemanticType }};

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
        struct_name: &str,
        struct_type: StructType<'ctxt>,
        field_types: &[SemanticType]
    ) -> Result<Option<FunctionValue<'ctxt>>, CodeGenError> {
        if field_types.iter().all(|t| !t.can_be_owned()) {
            return Ok(None);
        }

        let copy_fn_type = self.void_type().fn_type(&[self.ptr_type().into()], false);
        let copy_fn_value = self.module.add_function(
            &format!("__ql__{}_copy", struct_name),
            copy_fn_type,
            None
        );
        let copy_entry = self.context.append_basic_block(copy_fn_value, "entry");
        self.builder.position_at_end(copy_entry);
        
        let struct_ptr = copy_fn_value.get_nth_param(0).unwrap().into_pointer_value();
        for (i, field_type) in field_types.iter().enumerate() {
            if field_type.can_be_owned() {
                let field_ptr = self.builder.build_struct_gep(
                    struct_type,
                    struct_ptr,
                    i as u32,
                    &format!("{}.{}", struct_name, i)
                )?;
                self.copy_value(field_ptr, field_type)?;
            }
        }
        
        self.builder.build_return(None)?;
        Ok(Some(copy_fn_value))
    }

    fn create_drop_fn(
        &self,
        struct_name: &str,
        struct_type: StructType<'ctxt>,
        field_types: &[SemanticType]
    ) -> Result<Option<FunctionValue<'ctxt>>, CodeGenError> {
        if field_types.iter().all(|t| !t.can_be_owned()) {
            return Ok(None);
        }

        let drop_fn_type = self.void_type().fn_type(&[self.ptr_type().into()], false);
        let drop_fn_value = self.module.add_function(
            &format!("__ql__{}_drop", struct_name),
            drop_fn_type,
            None
        );
        let drop_entry = self.context.append_basic_block(drop_fn_value, "entry");
        self.builder.position_at_end(drop_entry);
        
        let struct_ptr = drop_fn_value.get_nth_param(0).unwrap().into_pointer_value();
        for (i, field_type) in field_types.iter().enumerate() {
            if field_type.can_be_owned() {
                let field_ptr = self.builder.build_struct_gep(
                    struct_type,
                    struct_ptr,
                    i as u32,
                    &format!("{}.{}", struct_name, i)
                )?;
                self.drop_value(field_ptr, field_type)?;
            }
        }

        self.builder.build_return(None)?;
        Ok(Some(drop_fn_value))
    }

    fn create_get_nth_fn(
        &self,
        struct_name: &str,
        struct_type: StructType<'ctxt>,
        field_types: &[SemanticType]
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let get_nth_fn_type = self.ptr_type().fn_type(
            &[self.ptr_type().into(), self.int_type().into(), self.ptr_type().into()],
            false
        );
        let get_nth_fn = self.module.add_function(
            &format!("__ql__{}_get_nth", struct_name),
            get_nth_fn_type,
            None
        );
        let get_nth_entry = self.context.append_basic_block(get_nth_fn, "entry");
        self.builder.position_at_end(get_nth_entry);

        let struct_ptr = get_nth_fn.get_nth_param(0).unwrap().into_pointer_value();
        let index = get_nth_fn.get_nth_param(1).unwrap().into_int_value();
        let column_type_out = get_nth_fn.get_nth_param(2).unwrap().into_pointer_value();
        
        let mut switch_blocks = vec![];
        for (i, field_type) in field_types.iter().enumerate() {
            let field_index = self.context.i32_type().const_int(i as u64, false);
            let field_block = self.context.append_basic_block(get_nth_fn, &format!("field_{}", i));
            self.builder.position_at_end(field_block);
            
            let column_type: ColumnType = field_type.into();
            self.builder.build_store(
                column_type_out,
                self.context.i32_type().const_int(column_type as u64, false)
            )?;

            let field_ptr = self.builder.build_struct_gep(
                struct_type,
                struct_ptr,
                i as u32,
                &format!("{}.{}", struct_name, i)
            )?;
            self.builder.build_return(Some(&field_ptr))?;

            switch_blocks.push((field_index, field_block));
        }

        let unreachable_block = self.context.append_basic_block(get_nth_fn, "unreachable");
        self.builder.position_at_end(unreachable_block);
        self.builder.build_unreachable()?;

        self.builder.position_at_end(get_nth_entry);
        self.builder.build_switch(index, unreachable_block, &switch_blocks)?;
        Ok(get_nth_fn)
    }

    fn create_set_nth_fn(
        &self,
        struct_name: &str,
        struct_type: StructType<'ctxt>,
        field_types: &[SemanticType]
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let set_nth_fn_type = self.void_type().fn_type(
            &[self.ptr_type().into(), self.int_type().into(), self.ptr_type().into()],
            false
        );
        let set_nth_fn = self.module.add_function(
            &format!("__ql__{}_set_nth", struct_name),
            set_nth_fn_type,
            None
        );
        let set_nth_entry = self.context.append_basic_block(set_nth_fn, "entry");
        self.builder.position_at_end(set_nth_entry);

        let struct_ptr = set_nth_fn.get_nth_param(0).unwrap().into_pointer_value();
        let index = set_nth_fn.get_nth_param(1).unwrap().into_int_value();
        let value_ptr = set_nth_fn.get_nth_param(2).unwrap().into_pointer_value();

        let mut switch_blocks = vec![];
        for (i, field_type) in field_types.iter().enumerate() {
            let field_index = self.context.i32_type().const_int(i as u64, false);
            let field_block = self.context.append_basic_block(set_nth_fn, &format!("field_{}", i));
            self.builder.position_at_end(field_block);
            
            let field_ptr = self.builder.build_struct_gep(
                struct_type,
                struct_ptr,
                i as u32,
                &format!("{}.{}", struct_name, i)
            )?;
            let llvm_value = self.builder.build_load(
                self.llvm_basic_type(field_type),
                value_ptr,
                &format!("load_value_{}", i)
            )?;
            self.builder.build_store(field_ptr, llvm_value)?;
            self.builder.build_return(None)?;
            switch_blocks.push((field_index, field_block));
        }

        let unreachable_block = self.context.append_basic_block(set_nth_fn, "unreachable");
        self.builder.position_at_end(unreachable_block);
        self.builder.build_unreachable()?;

        self.builder.position_at_end(set_nth_entry);
        self.builder.build_switch(index, unreachable_block, &switch_blocks)?;
        Ok(set_nth_fn)
    }

    pub(super) fn gen_struct_info(
        &mut self,
        name: &str,
        field_types: &[SemanticType],
        gen_field_access: bool
    ) -> Result<GenStructInfo<'ctxt>, CodeGenError> {
        let llvm_field_types = field_types.iter()
            .map(|field_type| self.llvm_basic_type(field_type))
            .collect::<Vec<BasicTypeEnum>>();
        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&llvm_field_types, false);

        let num_fields = field_types.len() as u32;
        let copy_fn = self.create_copy_fn(name, struct_type, field_types)?;
        let drop_fn = self.create_drop_fn(name, struct_type, field_types)?;
        let (get_nth_fn_ptr, set_nth_fn_ptr) = if gen_field_access {
            let get_nth_fn = self.create_get_nth_fn(name, struct_type, field_types)?;
            let set_nth_fn = self.create_set_nth_fn(name, struct_type, field_types)?;
            (
                get_nth_fn.as_global_value().as_pointer_value(),
                set_nth_fn.as_global_value().as_pointer_value()
            )
        } else {
            (self.ptr_type().const_null(), self.ptr_type().const_null())
        };

        let copy_fn_ptr = match copy_fn {
            Some(f) => f.as_global_value().as_pointer_value(),
            None => self.ptr_type().const_null(),
        };
        let drop_fn_ptr = match drop_fn {
            Some(f) => f.as_global_value().as_pointer_value(),
            None => self.ptr_type().const_null(),
        };

        let type_info_value = self.runtime.type_info_type.const_named_struct(&[
            struct_type.size_of().unwrap().into(),
            self.int_type().const_int(num_fields as u64, false).into(),
            copy_fn_ptr.into(),
            drop_fn_ptr.into(),
            get_nth_fn_ptr.into(),
            set_nth_fn_ptr.into(),
        ]);
        let type_info = self.module.add_global(
            self.runtime.type_info_type,
            Some(AddressSpace::default()),
            &format!("__ql__{}_type_info", name)
        );
        type_info.set_initializer(&type_info_value);
        type_info.set_constant(true);

        Ok(GenStructInfo {
            struct_type,
            copy_fn,
            drop_fn,
            type_info,
        })
    }

    pub fn gen_struct(&mut self, sem_struct: &SemanticStruct) -> Result<(), CodeGenError> {  
        let field_types = sem_struct.field_order.iter()
            .map(|field_name| sem_struct.fields[field_name].clone())
            .collect::<Vec<SemanticType>>();
        let struct_info = self.gen_struct_info(&sem_struct.name, &field_types, true)?;
        self.struct_info.insert(sem_struct.id, struct_info);
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
        let struct_ptr = self.build_alloca(struct_info.struct_type.into(), &format!("{}_store", sem_struct.name))?;
        for (column_name, column_value) in sem_struct.field_order.iter().zip(column_values) {
            let column_type = &sem_struct.fields[column_name];
            let column_index = sem_struct.field_order.iter()
                .position(|x| x == column_name).unwrap() as u32;
            let column_ptr = self.builder.build_struct_gep(
                struct_info.struct_type,
                struct_ptr, 
                column_index,
                &format!("{}.{}", sem_struct.name, column_name)
            )?;

            self.builder.build_store(column_ptr, column_value.as_llvm_basic_value())?;
            if column_value.ownership() == Ownership::Borrowed {
                self.copy_value(column_ptr, column_type)?;
            }
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