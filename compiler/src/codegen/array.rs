use core::panic;

use inkwell::{types::BasicType, values::{AnyValue}, IntPredicate};

use crate::semantics::{Ownership, SemanticType, SemanticTypeKind};

use super::*;

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_array(&mut self, elem_exprs: &[SemanticExpression], elem_type: &SemanticType) -> Result<GenValue<'ctxt>, CodeGenError> {
        let elems = elem_exprs.iter()
            .map(|expr| self.gen_eval(expr))
            .collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;

        let llvm_elem_type = self.llvm_basic_type(&elem_type);
        let type_info = self.get_type_info(elem_type).as_pointer_value();
        let num_elems = elems.len();
        if num_elems == 0 {
            // Create an empty array
            let null_ptr = self.context.ptr_type(Default::default()).const_null();
            let zero = self.context.i32_type().const_zero();
            
            let array_ptr = self.builder.build_call(
                self.runtime.new_array,
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
            let elem_basic = elem.as_llvm_basic_value();
            let index = self.context.i32_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    array_type,
                    array_alloca,
                    &[self.context.i32_type().const_zero(), index],
                    &format!("elem_ptr_{}", i)
                )?
            };
            self.builder.build_store(elem_ptr, elem_basic)?;
            if elem.ownership() == Ownership::Borrowed {
                self.copy_value(elem_ptr, elem_type)?;
            }
        }

        // Call __ql__QLArray_new
        let num_elems = self.context.i32_type().const_int(num_elems as u64, false);
        let array_ptr = self.builder.build_call(
            self.runtime.new_array,
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
            self.runtime.index_array,
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
            self.runtime.array_length,
            &[array_ptr.into()],
            "array_length"
        )?.as_any_value_enum().into_int_value();

        Ok(GenValue::Integer(length_value))
    }

    pub fn gen_array_append(&self, array: GenValue<'ctxt>, elem: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, elem_type, .. } = array else {
            panic!("Expected array value");
        };

        let elem_ptr = self.builder.build_alloca(
            self.llvm_basic_type(&elem_type),
            "append_elem_ptr"
        )?;
        self.builder.build_store(elem_ptr, elem.as_llvm_basic_value())?;
        if elem.ownership() == Ownership::Borrowed {
            self.copy_value(elem_ptr, &elem_type)?;
        }

        self.builder.build_call(
            self.runtime.append_array,
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
            self.runtime.pop_array,
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

    pub fn gen_array_iter(&self, array: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Array { value: array_ptr, elem_type, .. } = array else {
            panic!("Expected array value");
        };

        let iter_ptr = self.builder.build_call(
            self.runtime.array_iter,
            &[array_ptr.into()],
            "array_iter"
        )?.as_any_value_enum().into_pointer_value();

        Ok(GenValue::Iterator {
            value: iter_ptr,
            elem_type,
            ownership: Ownership::Owned,
        })
    }

    pub fn gen_iterator_next(&self, iterator: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Iterator { value: iter_ptr, elem_type, .. } = iterator else {
            panic!("Expected iterator value");
        };

        let elem_ptr = self.builder.build_call(
            self.runtime.iterator_next,
            &[iter_ptr.into()],
            "iterator_next"
        )?.as_any_value_enum().into_pointer_value();

        let loaded_elem = self.builder.build_load(
            self.llvm_basic_type(&elem_type),
            elem_ptr,
            "iter_elem_load"
        )?;

        let ownership = if elem_type.can_be_owned() {
            Ownership::Borrowed
        } else {
            Ownership::Trivial
        };

        Ok(GenValue::new(&elem_type, loaded_elem, ownership))
    }

    pub fn gen_iterator_has_next(&self, iterator: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Iterator { value: iter_ptr, .. } = iterator else {
            panic!("Expected iterator value");
        };

        let has_next = self.builder.build_call(
            self.runtime.iterator_has_next,
            &[iter_ptr.into()],
            "iterator_has_next"
        )?.as_any_value_enum().into_int_value();

        Ok(GenValue::Bool(has_next))
    }

    pub fn gen_range(
        &mut self,
        start: Option<&SemanticExpression>,
        end: Option<&SemanticExpression>,
        inclusive: bool,
        step: Option<&SemanticExpression>
    ) -> Result<GenValue<'ctxt>, CodeGenError> {
        let zero = self.int_type().const_zero();
        let one = self.int_type().const_int(1, false);
        let neg_one = self.int_type().const_int((-1i64) as u64, true);
        let int_max = self.int_type().const_int(i32::MAX as u64, false);

        let start_val = match start {
            Some(expr) => {
                let val = self.gen_eval(expr)?;
                let GenValue::Integer(int_val) = val else {
                    panic!("Expected integer range start");
                };
                int_val
            }
            None => zero,
        };

        let end_val = match end {
            Some(expr) => {
                let val = self.gen_eval(expr)?;
                let GenValue::Integer(int_val) = val else {
                    panic!("Expected integer range end");
                };
                int_val
            }
            None => int_max,
        };

        let step_val = match step {
            Some(expr) => {
                let val = self.gen_eval(expr)?;
                let GenValue::Integer(int_val) = val else {
                    panic!("Expected integer range step");
                };
                int_val
            }
            None => one,
        };

        let end_val = if inclusive && end.is_some() {
            let is_pos = self.builder.build_int_compare(
                IntPredicate::SGT,
                step_val,
                zero,
                "range_step_pos"
            )?;
            let delta = self.builder.build_select(is_pos, one, neg_one, "range_delta")?
                .into_int_value();
            self.builder.build_int_add(end_val, delta, "range_end_adj")?
        } else {
            end_val
        };

        let iter_ptr = self.builder.build_call(
            self.runtime.iterator_range,
            &[start_val.into(), end_val.into(), step_val.into()],
            "iterator_range"
        )?.as_any_value_enum().into_pointer_value();

        Ok(GenValue::Iterator {
            value: iter_ptr,
            elem_type: SemanticType::new(SemanticTypeKind::Integer),
            ownership: Ownership::Owned,
        })
    }

    pub fn gen_iterator_collect(&self, iterator: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        let GenValue::Iterator { value: iter_ptr, elem_type, .. } = iterator else {
            panic!("Expected iterator value");
        };

        let array_ptr = self.builder.build_call(
            self.runtime.iterator_collect,
            &[iter_ptr.into()],
            "iterator_collect"
        )?.as_any_value_enum().into_pointer_value();

        Ok(GenValue::Array {
            value: array_ptr,
            elem_type,
            ownership: Ownership::Owned,
        })
    }
}