use core::panic;

use inkwell::{types::BasicType, values::{AnyValue}};

use crate::semantics::{Ownership, SemanticType, SemanticTypeKind};

use super::*;

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_array(&mut self, elem_exprs: &[SemanticExpression], elem_type: &SemanticType) -> Result<GenValue<'ctxt>, CodeGenError> {
        let elems = elem_exprs.iter()
            .map(|expr| self.gen_eval(expr))
            .collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;

        // Get the LLVM type for the array elements
        let llvm_elem_type = self.llvm_basic_type(&elem_type);

        let type_info = match elem_type.kind() {
            SemanticTypeKind::Integer => self.runtime_functions.int_type_info.as_pointer_value(),
            SemanticTypeKind::Bool => self.runtime_functions.bool_type_info.as_pointer_value(),
            SemanticTypeKind::String => self.runtime_functions.string_type_info.as_pointer_value(),
            SemanticTypeKind::NamedStruct(struct_id, _) => self.struct_info[&struct_id].type_info.as_pointer_value(),
            SemanticTypeKind::Array(_) => self.runtime_functions.array_type_info.as_pointer_value(),
            _ => self.ptr_type().const_null(),
        };
        
        let num_elems = elems.len();
        if num_elems == 0 {
            // Create an empty array
            let null_ptr = self.context.ptr_type(Default::default()).const_null();
            let zero = self.context.i32_type().const_zero();
            
            let array_ptr = self.builder.build_call(
                self.runtime_functions.new_array.into(),
                &[null_ptr.into(), zero.into(), type_info.into()],
                "empty_array"
            )?.as_any_value_enum().into_pointer_value();
            
            return Ok(GenValue::Array {
                value: array_ptr,
                elem_type: elem_type.clone(),
                ownership: Ownership::Owned,
            });
        }

        // Allocate memory for the elements array
        let array_type = llvm_elem_type.array_type(num_elems as u32);
        let array_alloca = self.builder.build_alloca(array_type, "array_elems")?;

        // Store each element in the array
        for (i, elem) in elems.into_iter().enumerate() {
            self.add_ref(&elem)?;
            let elem_basic = elem.as_llvm_basic_value();
            let index = self.context.i32_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    array_type,
                    array_alloca,
                    &[self.context.i32_type().const_int(0, false), index],
                    &format!("elem_ptr_{}", i)
                )?
            };
            self.builder.build_store(elem_ptr, elem_basic)?;
        }

        // Call __ql__QLArray_new
        let num_elems = self.context.i32_type().const_int(num_elems as u64, false);
        let array_ptr = self.builder.build_call(
            self.runtime_functions.new_array.into(),
            &[array_alloca.into(), num_elems.into(), type_info.into()],
            "array_alloc"
        )?.as_any_value_enum().into_pointer_value();

        Ok(GenValue::Array {
            value: array_ptr,
            elem_type: elem_type.clone(),
            ownership: Ownership::Owned,
        })
    }

    pub fn gen_array_index(&self, array: GenValue<'ctxt>, index: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, elem_type, .. } = array else {
            panic!("Expected array value");
        };

        let elem_ptr = self.builder.build_call(
            self.runtime_functions.index_array.into(),
            &[array_ptr.into(), index.as_llvm_basic_value().into()],
            "array_index"
        )?.as_any_value_enum().into_pointer_value();

        let loaded_elem = self.builder.build_load(
            self.llvm_basic_type(&elem_type),
            elem_ptr,
            "load_array_elem"
        )?;

        Ok(GenValue::new(&elem_type, loaded_elem, Ownership::Borrowed))
    }

    pub fn gen_array_length(&self, array: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, .. } = array else {
            panic!("Expected array value");
        };

        let length_value = self.builder.build_call(
            self.runtime_functions.array_length.into(),
            &[array_ptr.into()],
            "array_length"
        )?.as_any_value_enum().into_int_value();

        Ok(GenValue::Integer(length_value))
    }

    pub fn gen_array_append(&self, array: GenValue<'ctxt>, elem: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, elem_type, .. } = array else {
            panic!("Expected array value");
        };

        self.add_ref(&elem)?;
        let elem_ptr = self.builder.build_alloca(
            self.llvm_basic_type(&elem_type),
            "append_elem_ptr"
        )?;
        self.builder.build_store(elem_ptr, elem.as_llvm_basic_value())?;

        self.builder.build_call(
            self.runtime_functions.append_array.into(),
            &[array_ptr.into(), elem_ptr.into()],
            "array_append"
        )?;

        Ok(GenValue::Void)
    }

    pub fn gen_array_pop(&self, array: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, elem_type, .. } = array else {
            panic!("Expected array value");
        };

        let elem_ptr = self.builder.build_call(
            self.runtime_functions.pop_array.into(),
            &[array_ptr.into()],
            "array_pop"
        )?.as_any_value_enum().into_pointer_value();

        let loaded_elem = self.builder.build_load(
            self.llvm_basic_type(&elem_type),
            elem_ptr,
            "pop_elem_load"
        )?;

        Ok(GenValue::new(&elem_type, loaded_elem, Ownership::Owned))
    }
}