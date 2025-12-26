use inkwell::{types::BasicType, values::{AnyValue, BasicValueEnum}};

use super::{CodeGen, CodeGenError, QLType, QLValue};

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_array(&self, elems: Vec<QLValue<'ctxt>>, elem_type: &QLType) -> Result<QLValue<'ctxt>, CodeGenError> {
        // Get the LLVM type for the array elements
        let llvm_elem_type = self.try_get_nonvoid_type(elem_type)?;
        let elem_size = llvm_elem_type.size_of().unwrap();
        
        if elems.is_empty() {
            // Create an empty array
            let null_ptr = self.context.ptr_type(Default::default()).const_null();
            let zero = self.context.i32_type().const_zero();
            
            let array_ptr = self.builder.build_call(
                self.runtime_functions.new_array.into(),
                &[null_ptr.into(), zero.into(), elem_size.into()],
                "empty_array"
            )?.as_any_value_enum().into_pointer_value();
            
            return Ok(QLValue::Array(array_ptr, elem_type.clone(), true));
        }

        // Allocate memory for the elements array
        let array_type = llvm_elem_type.array_type(elems.len() as u32);
        let array_alloca = self.builder.build_alloca(array_type, "array_elems")?;

        // Store each element in the array
        for (i, elem) in elems.iter().enumerate() {
            let elem_basic: BasicValueEnum = elem.clone().try_into()?;
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
        let num_elems = self.context.i32_type().const_int(elems.len() as u64, false);
        let array_ptr = self.builder.build_call(
            self.runtime_functions.new_array.into(),
            &[array_alloca.into(), num_elems.into(), elem_size.into()],
            "array_alloc"
        )?.as_any_value_enum().into_pointer_value();

        Ok(QLValue::Array(array_ptr, elem_type.clone(), true))
    }

    pub fn gen_array_index(&self, array: QLValue<'ctxt>, index: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let QLValue::Array(array_ptr, elem_type, _) = array {
            let index_val = match index {
                QLValue::Integer(int_val) => int_val,
                _ => return Err(CodeGenError::UnexpectedTypeError),
            };

            let elem_ptr = self.builder.build_call(
                self.runtime_functions.index_array.into(),
                &[array_ptr.into(), index_val.into()],
                "array_index"
            )?.as_any_value_enum().into_pointer_value();

            let loaded_elem = self.builder.build_load(
                self.try_get_nonvoid_type(&elem_type)?,
                elem_ptr,
                "load_array_elem"
            )?;

            Ok(elem_type.to_value(loaded_elem, true))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}