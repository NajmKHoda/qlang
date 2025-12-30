use inkwell::values::{BasicValueEnum, IntValue, PointerValue, StructValue};

use crate::{codegen::QLScope, tokens::ExpressionNode};

use super::{CodeGen, CodeGenError};

#[derive(Clone, PartialEq, Debug)]
pub enum QLType {
    Integer,
    Bool,
    String,
    Array(Box<QLType>),
    Table(String),
    Void
}

impl QLType {
    pub(super) fn to_value<'a>(&self, value: BasicValueEnum<'a>, is_owned: bool) -> QLValue<'a> {
        match self {
            QLType::Integer => QLValue::Integer(value.into_int_value()),
            QLType::Bool => QLValue::Bool(value.into_int_value()),
            QLType::String => QLValue::String(value.into_pointer_value(), is_owned),
            QLType::Array(array_type) => QLValue::Array(value.into_pointer_value(), *array_type.clone(), is_owned),
            QLType::Table(table_name) => QLValue::TableRow(value.into_struct_value(), table_name.clone(), is_owned),
            QLType::Void => panic!("Mismatch between void type and basic value"),
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            QLType::Integer => true,
            QLType::Bool => true,
            QLType::Void => true,
            _ => false
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    String(PointerValue<'a>, bool),
    Array(PointerValue<'a>, QLType, bool),
    TableRow(StructValue<'a>, String, bool),
    Void
}

impl<'a> QLValue<'a> {
    pub fn get_type(&self) -> QLType {
        match self {
            QLValue::Integer(_) => QLType::Integer,
            QLValue::Bool(_) => QLType::Bool,
            QLValue::String(_, _) => QLType::String,
            QLValue::Array(_, elem_type, _) => QLType::Array(Box::new(elem_type.clone())),
            QLValue::TableRow(_, table_name, _) => QLType::Table(table_name.clone()),
            QLValue::Void => QLType::Void
        }
    }

    pub fn is_primitive(&self) -> bool {
        return self.get_type().is_primitive();
    }

    pub fn is_owned(&self) -> bool {
        match self {
            QLValue::String(_, is_owned) => *is_owned,
            QLValue::Array(_, _, is_owned) => *is_owned,
            QLValue::TableRow(_, _, is_owned) => *is_owned,
            _ => true
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
            QLValue::Array(arr_val, _, _) => Ok(BasicValueEnum::PointerValue(arr_val)),
            QLValue::TableRow(struct_val, _, _) => Ok(BasicValueEnum::StructValue(struct_val)),
            QLValue::Void => Err(CodeGenError::UnexpectedTypeError),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn add_ref(&self, val: &QLValue<'ctxt>) -> Result<(), CodeGenError> {
        match val {
            QLValue::String(str_ptr, true) => {
                self.builder.build_call(
                    self.runtime_functions.add_string_ref.into(),
                    &[(*str_ptr).into()],
                    "add_string_ref"
                )?;
            }
            QLValue::Array(array_ptr, _, true) => {
                self.builder.build_call(
                    self.runtime_functions.add_array_ref.into(),
                    &[(*array_ptr).into()],
                    "add_array_ref"
                )?;
            }
            QLValue::TableRow(struct_value, table_name, true) => {
                if let Some(ref copy_fn) = self.get_table(table_name)?.copy_fn {
                    self.builder.build_call(
                        copy_fn.llvm_function,
                        &[(*struct_value).into()],
                        "table_copy"
                    )?;
                }
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn remove_ref(&self, val: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        match val {
            QLValue::String(str_ptr, _) => {
                self.builder.build_call(
                    self.runtime_functions.remove_string_ref.into(),
                    &[str_ptr.into()],
                    "remove_string_ref"
                )?;
            }
            QLValue::Array(array_ptr, _, _) => {
                self.builder.build_call(
                    self.runtime_functions.remove_array_ref.into(),
                    &[array_ptr.into()],
                    "remove_array_ref"
                )?;
            }
            QLValue::TableRow(struct_value, table_name, _) => {
                if let Some(ref drop_fn) = self.get_table(&table_name)?.drop_fn {
                    self.builder.build_call(
                        drop_fn.llvm_function,
                        &[struct_value.into()],
                        "table_row_drop"
                    )?;
                }
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn remove_if_temp(&self, val: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if !val.is_primitive() && !val.is_owned() {
            self.remove_ref(val)?;
        }
        Ok(())
    }

    pub(super) fn release_scope(&self, scope: &QLScope<'ctxt>) -> Result<(), CodeGenError> {
        for (name, var) in &scope.vars {
            if !var.ql_type.is_primitive() {
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

    pub fn gen_lone_expression(&mut self, expr: &Box<ExpressionNode>) -> Result<(), CodeGenError> {
        let val = expr.gen_eval(self)?;
        self.remove_ref(val)?;
        Ok(())
    }
}