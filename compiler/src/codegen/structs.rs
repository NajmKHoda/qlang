use std::{collections::HashMap};

use inkwell::{AddressSpace, types::{BasicTypeEnum, IntType, StructType}, values::{ArrayValue, FunctionValue, GlobalValue, IntValue, PointerValue, StructValue}};

use crate::{codegen::{CodeGenError, data::GenValue}, semantics::{Ownership, SemanticExpression, SemanticStruct, SemanticType, SemanticTypeKind}};

use super::CodeGen;

pub(super) struct GenStructInfo<'a> {
    pub(super) struct_type: StructType<'a>,
    pub(super) type_info: GlobalValue<'a>,
    pub(super) copy_fn: Option<FunctionValue<'a>>,
    pub(super) drop_fn: Option<FunctionValue<'a>>,
}

impl SemanticType {
    pub(super) fn to_type_enum<'a>(&self, int_type: IntType<'a>) -> IntValue<'a> {
        let enum_value = match self.kind() {
            SemanticTypeKind::Integer => 0,
            SemanticTypeKind::Bool => 1,
            SemanticTypeKind::String => 2,
            SemanticTypeKind::Array(_) => 3,
            _ => panic!("Unsupported type for type enum conversion"),
        };
        int_type.const_int(enum_value, false)
    }
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
    ) -> Result<PointerValue<'ctxt>, CodeGenError> {
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
        Ok(elem_drop_fn.as_global_value().as_pointer_value().into())
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
        
        let num_fields = sem_struct.field_order.len() as u32;
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

        let elem_drop_fn_ptr = match drop_fn {
            Some(drop_fn) => self.create_elem_drop_fn(sem_struct, struct_type, drop_fn)?,
            None => self.ptr_type().const_null().into(),
        };

        let fields_arr: ArrayValue = self.runtime.struct_field_type
            .const_array(&sem_struct.field_order.iter().enumerate()
            .map(|(i, field_name)| {
                let field_type = &sem_struct.fields[field_name];
                let offset = self.target_data.offset_of_element(&struct_type, i as u32).unwrap();
                self.runtime.struct_field_type.const_named_struct(&[
                    field_type.to_type_enum(self.int_type()).into(),
                    self.context.i32_type().const_int(offset, false).into(),
                ])
            })
            .collect::<Vec<StructValue>>());
        let fields_global = self.module.add_global(
            self.runtime.struct_field_type.array_type(num_fields),
            Some(AddressSpace::default()),
            &format!("__ql__{}_fields", sem_struct.name)
        );
        fields_global.set_initializer(&fields_arr);
        fields_global.set_constant(true);

        let type_info_value = self.runtime.type_info_type.const_named_struct(&[
            struct_type.size_of().unwrap().into(),
            elem_drop_fn_ptr.into(),
            self.context.i32_type().const_int(num_fields as u64, false).into(),
            fields_global.as_pointer_value().into(),
        ]);
        let type_info_global = self.module.add_global(
            self.runtime.type_info_type,
            Some(AddressSpace::default()),
            &format!("__ql__{}_type_info", sem_struct.name)
        );
        type_info_global.set_initializer(&type_info_value);
        type_info_global.set_constant(true);

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