use inkwell::values::{AnyValue, BasicValueEnum, IntValue, PointerValue};

use super::{CodeGen, CodeGenError};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QLType {
    Integer,
    Bool,
    String,
    Void
}

impl QLType {
    pub(super) fn to_value<'a>(self, value: BasicValueEnum<'a>, is_owned: bool) -> QLValue<'a> {
        match self {
            QLType::Integer => QLValue::Integer(value.into_int_value()),
            QLType::Bool => QLValue::Bool(value.into_int_value()),
            QLType::String => QLValue::String(value.into_pointer_value(), is_owned),
            QLType::Void => panic!("Mismatch between void type and basic value"),
        }
    }
}

pub(super) struct QLVariable<'a> {
    pub(super) ql_type: QLType,
    pointer: PointerValue<'a>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    String(PointerValue<'a>, bool),
    Void
}

impl<'a> QLValue<'a> {
    pub fn get_type(&self) -> QLType {
        match self {
            QLValue::Integer(_) => QLType::Integer,
            QLValue::Bool(_) => QLType::Bool,
            QLValue::String(_, _) => QLType::String,
            QLValue::Void => QLType::Void
        }
    }
}

impl<'a> TryFrom<QLValue<'a>> for BasicValueEnum<'a> {
    type Error = CodeGenError;

    fn try_from(value: QLValue<'a>) -> Result<Self, Self::Error> {
        match value {
            QLValue::Integer(int_val) => Ok(BasicValueEnum::IntValue(int_val)),
            QLValue::Bool(int_val) => Ok(BasicValueEnum::IntValue(int_val)),
            QLValue::String(str_val, _) => Ok(BasicValueEnum::PointerValue(str_val)),
            QLValue::Void => Err(CodeGenError::UnexpectedTypeError),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    fn get_var<'a>(&'a self, name: &str) -> Option<&'a QLVariable<'ctxt>> {
        for scope in self.vars.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub(super) fn add_ref(&self, val: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let QLValue::String(str_ptr, true) = val {
            self.builder.build_call(
                self.runtime_functions.add_string_ref.into(),
                &[str_ptr.into()],
                "add_string_ref"
            )?;
        }
        Ok(())
    }

    pub(super) fn remove_ref(&self, val: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let QLValue::String(str_ptr, _) = val {
            self.builder.build_call(
                self.runtime_functions.remove_string_ref.into(),
                &[str_ptr.into()],
                "remove_string_ref"
            )?;
        }
        Ok(())
    }

    pub(super) fn remove_if_temp(&self, val: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let QLValue::String(_, false) = val {
            self.remove_ref(val)?;
        }
        Ok(())
    }

    pub(super) fn remove_var_refs(&self) -> Result<(), CodeGenError> {
        for (name, var) in self.vars.last().unwrap() {
            if var.ql_type == QLType::String {
                let loaded_val = self.load_var(name)?;
                self.remove_ref(loaded_val)?;
            }
        }
        Ok(())
    }

    pub fn const_int(&self, value: i32) -> QLValue<'ctxt> {
        QLValue::Integer(self.int_type().const_int(value as u64, false))
    }

    pub fn const_bool(&self, value: bool) -> QLValue<'ctxt> {
        QLValue::Bool(self.bool_type().const_int(value as u64, false))
    }

    pub fn const_str(&self, value: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        let raw_str = self.builder.build_global_string_ptr(value, "global_str")?.as_pointer_value();
        let length = self.context.i32_type().const_int(value.len() as u64, false);
        let str_ptr = self.builder.build_call(
            self.runtime_functions.new_string.into(),
            &[raw_str.into(), length.into(), self.bool_type().const_int(1, false).into()],
            "string_alloc"
        )?.as_any_value_enum().into_pointer_value();
        Ok(QLValue::String(str_ptr, false))
    }

    pub fn load_var(&self, name: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(variable) = self.get_var(name) {
            let var_type = self.try_get_nonvoid_type(&variable.ql_type)?;
            let res: BasicValueEnum = self.builder.build_load(var_type, variable.pointer, "load")?;
            Ok(variable.ql_type.to_value(res, true))
        } else if let Some(arg) = self.cur_fn().try_get_arg_value(name) {
            Ok(arg)
        } else {
            Err(CodeGenError::UndefinedVariableError(name.to_string()))
        }
    }

    pub fn define_var(&mut self, name: &str, var_type: QLType, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        let llvm_type = self.try_get_nonvoid_type(&var_type)?;
        let cur_scope = self.vars.last_mut().unwrap();
        if cur_scope.contains_key(name) {
            return Err(CodeGenError::DuplicateDefinitionError(name.to_string()));
        } else if var_type == QLType::Void || var_type != value.get_type() {
            return Err(CodeGenError::UnexpectedTypeError);
        }

        let pointer = self.builder.build_alloca(llvm_type, name)?;
        self.builder.build_store::<BasicValueEnum>(pointer, value.try_into()?)?;
        let var = QLVariable {
            ql_type: var_type,
            pointer
        };
        cur_scope.insert(name.to_string(), var);
        
        Ok(())
    }

    pub fn store_var(&self, name: &str, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let Some(variable) = self.get_var(name) {
            if variable.ql_type != value.get_type() {
                return Err(CodeGenError::UnexpectedTypeError);
            }
            self.add_ref(value)?;
            self.builder.build_store::<BasicValueEnum>(variable.pointer, value.try_into()?)?;
            Ok(())
        } else if let Some(_) = self.cur_fn().try_get_arg_value(name) {
            Err(CodeGenError::BadArgumentMutationError(self.cur_fn().name.clone(), name.to_string()))
        } else {
            Err(CodeGenError::UndefinedVariableError(name.to_string()))
        }
    }

    
}