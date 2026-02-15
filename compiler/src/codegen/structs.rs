use std::{collections::HashMap};

use inkwell::{AddressSpace, types::{BasicTypeEnum, StructType}, values::{ArrayValue, FunctionValue, GlobalValue, StructValue}};

use crate::{codegen::{CodeGenError, data::GenValue, runtime::QLType}, semantics::{Ownership, SemanticExpression, SemanticStruct, SemanticType }};

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

    pub(super) fn gen_struct_type_info(&mut self, name: &str, field_types: &[&SemanticType])
        -> Result<(StructType<'ctxt>, GlobalValue<'ctxt>), CodeGenError>
    {
        let llvm_field_types = field_types.iter()
            .map(|ty| self.llvm_basic_type(ty))
            .collect::<Vec<BasicTypeEnum>>();

        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&llvm_field_types, false);
        let num_fields = field_types.len() as u32;
        let fields_arr: ArrayValue = self.runtime.struct_field_type
            .const_array(&field_types.iter().enumerate()
            .map(|(i, field_type)| {
                let offset = self.target_data.offset_of_element(&struct_type, i as u32).unwrap();
                self.runtime.struct_field_type.const_named_struct(&[
                    self.int_type().const_int(offset, false).into(),
                    self.get_type_info(field_type).as_pointer_value().into(),
                ])
            })
            .collect::<Vec<StructValue>>());
        let fields_global = self.module.add_global(
            self.runtime.struct_field_type.array_type(num_fields),
            Some(AddressSpace::default()),
            &format!("__ql__{}_fields", name)
        );
        fields_global.set_initializer(&fields_arr);
        fields_global.set_constant(true);

        let type_info_value = self.runtime.type_info_type.const_named_struct(&[
            self.int_type().const_int(QLType::Struct as u64, false).into(),
            struct_type.size_of().unwrap().into(),
            self.int_type().const_int(num_fields as u64, false).into(),
            fields_global.as_pointer_value().into(),
        ]);
        let type_info_global = self.module.add_global(
            self.runtime.type_info_type,
            Some(AddressSpace::default()),
            &format!("__ql__{}_type_info", name)
        );
        type_info_global.set_initializer(&type_info_value);
        type_info_global.set_constant(true);

        Ok((struct_type, type_info_global))
    }

    pub fn gen_struct(&mut self, sem_struct: &SemanticStruct) -> Result<(), CodeGenError> {        
        let (struct_type, type_info_global) = self.gen_struct_type_info(
            &sem_struct.name,
            sem_struct.field_order.iter()
                .map(|field_name| &sem_struct.fields[field_name])
                .collect::<Vec<&SemanticType>>().as_slice()
        )?;
        
        let has_heap_fields = sem_struct.fields
            .values()
            .any(|field_type| field_type.can_be_owned());
        let (copy_fn, drop_fn) = if has_heap_fields {
            let copy_fn = self.create_copy_fn(sem_struct, struct_type)?;
            let drop_fn = self.create_drop_fn(sem_struct, struct_type)?;
            (Some(copy_fn), Some(drop_fn))
        } else {
            (None, None)
        };

        self.struct_info.insert(sem_struct.id, GenStructInfo {
            struct_type,
            copy_fn,
            drop_fn,
            type_info: type_info_global,
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