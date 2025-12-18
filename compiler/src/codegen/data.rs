use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use super::{CodeGen, CodeGenError};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QLType {
    Integer,
    Bool,
    Void
}

pub(super) struct QLVariable<'a> {
    ql_type: QLType,
    pointer: PointerValue<'a>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    Void
}

impl<'a> QLValue<'a> {
    pub fn get_type(&self) -> QLType {
        match self {
            QLValue::Integer(_) => QLType::Integer,
            QLValue::Bool(_) => QLType::Bool,
            QLValue::Void => QLType::Void
        }
    }
}

impl<'a> TryFrom<BasicValueEnum<'a>> for QLValue<'a> {
    type Error = CodeGenError;

    fn try_from(value: BasicValueEnum<'a>) -> Result<Self, Self::Error> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                match int_val.get_type().get_bit_width() {
                    1 => Ok(QLValue::Bool(int_val)),
                    32 => Ok(QLValue::Integer(int_val)),
                    _ => Err(CodeGenError::UnexpectedTypeError),
                }
            }
            _ => Err(CodeGenError::UnexpectedTypeError),
        }
    }
}

impl<'a> TryFrom<QLValue<'a>> for BasicValueEnum<'a> {
    type Error = CodeGenError;

    fn try_from(value: QLValue<'a>) -> Result<Self, Self::Error> {
        match value {
            QLValue::Integer(int_val) => Ok(BasicValueEnum::IntValue(int_val)),
            QLValue::Bool(int_val) => Ok(BasicValueEnum::IntValue(int_val)),
            QLValue::Void => Err(CodeGenError::UnexpectedTypeError),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn const_int(&self, value: i32) -> QLValue<'ctxt> {
        QLValue::Integer(self.int_type().const_int(value as u64, false))
    }

    pub fn const_bool(&self, value: bool) -> QLValue<'ctxt> {
        QLValue::Bool(self.bool_type().const_int(value as u64, false))
    }

    pub fn load_var(&self, name: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(variable) = self.vars.get(name) {
            let var_type = self.try_get_nonvoid_type(&variable.ql_type)?;
            let res: QLValue<'ctxt> = self.builder.build_load(var_type, variable.pointer, "load")?.try_into()?;
            Ok(res)
        } else {
            Err(CodeGenError::UndefinedVariableError(name.to_string()))
        }
    }

    pub fn define_var(&mut self, name: &str, var_type: QLType, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if self.vars.contains_key(name) {
            return Err(CodeGenError::DuplicateDefinitionError(name.to_string()));
        } else if var_type == QLType::Void || var_type != value.get_type() {
            return Err(CodeGenError::UnexpectedTypeError);
        }
        
        let llvm_type = self.try_get_nonvoid_type(&var_type)?;
        let pointer = self.builder.build_alloca(llvm_type, name)?;
        self.builder.build_store::<BasicValueEnum>(pointer, value.try_into()?)?;

        let var = QLVariable {
            ql_type: var_type,
            pointer
        };
        self.vars.insert(name.to_string(), var);
        
        Ok(())
    }

    pub fn store_var(&self, name: &str, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let Some(variable) = self.vars.get(name) {
            if variable.ql_type != value.get_type() {
                return Err(CodeGenError::UnexpectedTypeError);
            }
            self.builder.build_store::<BasicValueEnum>(variable.pointer, value.try_into()?)?;
            Ok(())
        } else {
            Err(CodeGenError::UndefinedVariableError(name.to_string()))
        }
    }
}