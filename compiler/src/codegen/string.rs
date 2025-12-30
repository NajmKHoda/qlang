use inkwell::{AddressSpace, values::AnyValue};

use super::{CodeGen, CodeGenError, QLValue};

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn gen_const_strs(&self) -> Result<(), CodeGenError> {
        for (str_val, global_val) in &self.strings {
            let raw_str = self.builder.build_global_string_ptr(str_val, "raw_str")?.as_pointer_value();
            let ql_string_ptr = self.builder.build_call(
                self.runtime_functions.new_string.into(),
                &[
                    raw_str.into(),
                    self.int_type().const_int(str_val.len() as u64, false).into(),
                    self.bool_type().const_int(1, false).into()
                ],
                "ql_string"
            )?.as_any_value_enum().into_pointer_value();
            self.builder.build_store(global_val.as_pointer_value(), ql_string_ptr)?;
        }
        Ok(())
    }

    pub(super) fn drop_const_strs(&self) -> Result<(), CodeGenError> {
        for (_, global_val) in &self.strings {
            let str_ptr = self.builder.build_load(
                self.ptr_type(),
                global_val.as_pointer_value(),
                "const_str_load_for_drop"
            )?.into_pointer_value();
            self.builder.build_call(
                self.runtime_functions.remove_string_ref.into(),
                &[str_ptr.into()],
                "remove_const_str"
            )?;
        }
        Ok(())
    }

    pub fn const_str(&mut self, value: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        if !self.strings.contains_key(value) {
            let global_val = self.module.add_global(self.ptr_type(), Some(AddressSpace::default()), "const_str");
            global_val.set_initializer(&self.ptr_type().const_null());
            self.strings.insert(value.to_string(), global_val);
        }

        let global_ptr = self.strings[value].as_pointer_value();
        let str_ptr = self.builder.build_load(self.ptr_type(), global_ptr, "const_str_load")?.into_pointer_value();
        Ok(QLValue::String(str_ptr, true))
    }

    
}