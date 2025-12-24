use inkwell::values::{AnyValue, BasicValueEnum, IntValue, PointerValue};

use crate::{codegen::QLScope, tokens::ExpressionNode};

use super::{CodeGen, CodeGenError};

#[derive(Clone, PartialEq, Debug)]
pub enum QLType {
    Integer,
    Bool,
    String,
    Table(String),
    Void
}

impl QLType {
    pub(super) fn to_value<'a>(&self, value: BasicValueEnum<'a>, is_owned: bool) -> QLValue<'a> {
        match self {
            QLType::Integer => QLValue::Integer(value.into_int_value()),
            QLType::Bool => QLValue::Bool(value.into_int_value()),
            QLType::String => QLValue::String(value.into_pointer_value(), is_owned),
            QLType::Table(table_name) => QLValue::TableRow(value.into_pointer_value(), table_name.clone()),
            QLType::Void => panic!("Mismatch between void type and basic value"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    String(PointerValue<'a>, bool),
    TableRow(PointerValue<'a>, String),
    Void
}

impl<'a> QLValue<'a> {
    pub fn get_type(&self) -> QLType {
        match self {
            QLValue::Integer(_) => QLType::Integer,
            QLValue::Bool(_) => QLType::Bool,
            QLValue::String(_, _) => QLType::String,
            QLValue::TableRow(_, table_name) => QLType::Table(table_name.clone()),
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
            QLValue::TableRow(ptr_val, _) => Ok(BasicValueEnum::PointerValue(ptr_val)),
            QLValue::Void => Err(CodeGenError::UnexpectedTypeError),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn add_ref(&self, val: &QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let QLValue::String(str_ptr, true) = val {
            self.builder.build_call(
                self.runtime_functions.add_string_ref.into(),
                &[(*str_ptr).into()],
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

    pub(super) fn release_scope(&self, scope: &QLScope<'ctxt>) -> Result<(), CodeGenError> {
        for (name, var) in &scope.vars {
            if var.ql_type == QLType::String {
                let loaded_val = self.load_var(&name)?;
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

    pub fn gen_lone_expression(&mut self, expr: &Box<ExpressionNode>) -> Result<(), CodeGenError> {
        let val = expr.gen_eval(self)?;
        self.remove_ref(val)?;
        Ok(())
    }
}