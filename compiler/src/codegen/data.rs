use inkwell::{values::{BasicValueEnum, IntValue}};

use super::{CodeGen, CodeGenError};

#[derive(Clone, Copy, PartialEq)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    Void
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

    pub fn load_var(&self, name: &String) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(pointer) = self.vars.get(name) {
            let res = self.builder.build_load(self.int_type(), *pointer, "load").map(|v| v.into_int_value())?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UndefinedVariableError(name.clone()))
        }
    }

    pub fn store_var(&mut self, name: &String, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if !self.vars.contains_key(name) {
            // Allocate memory on first assignment
            let pointer = self.builder.build_alloca(self.int_type(), name)?;
            self.vars.insert(name.clone(), pointer);
        }
        
        let pointer = self.vars[name];
        self.builder.build_store::<BasicValueEnum>(pointer, value.try_into()?)?;
        Ok(())
    }
}