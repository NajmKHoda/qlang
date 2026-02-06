use inkwell::{types::BasicTypeEnum, values::{BasicValueEnum, IntValue, PointerValue, StructValue}};

use crate::semantics::{Ownership, SemanticType, SemanticTypeKind};

use super::{CodeGen, CodeGenError};

#[derive(Clone, PartialEq)]
pub enum GenValue<'a> {
    Integer(IntValue<'a>),
    Bool(IntValue<'a>),
    String {
        value: PointerValue<'a>,
        ownership: Ownership
    },
    Array {
        value: PointerValue<'a>,
        elem_type: SemanticType,
        ownership: Ownership
    },
    Struct {
        value: StructValue<'a>,
        struct_id: u32,
        ownership: Ownership
    },
    Callable(StructValue<'a>),
    Void
}

impl<'a> GenValue<'a> {
    pub fn new(sem_type: &SemanticType, llvm_value: BasicValueEnum<'a>, ownership: Ownership) -> Self {
        match sem_type.kind() {
            SemanticTypeKind::Integer => GenValue::Integer(llvm_value.into_int_value()),
            SemanticTypeKind::Bool => GenValue::Bool(llvm_value.into_int_value()),
            SemanticTypeKind::String => GenValue::String {
                value: llvm_value.into_pointer_value(),
                ownership: ownership
            },
            SemanticTypeKind::Array(elem_type) => GenValue::Array {
                value: llvm_value.into_pointer_value(),
                elem_type: elem_type,
                ownership: ownership
            },
            SemanticTypeKind::NamedStruct(struct_id, _) => GenValue::Struct {
                value: llvm_value.into_struct_value(),
                struct_id,
                ownership: ownership
            },
            SemanticTypeKind::Callable(_,_) => GenValue::Callable(llvm_value.into_struct_value()),
            SemanticTypeKind::Void => GenValue::Void,
            _ => panic!("Incomplete type found in semantic IR"),
        }
    }

    pub fn ownership(&self) -> Ownership {
        match self {
            GenValue::String { ownership, .. }
            | GenValue::Array { ownership, .. }
            | GenValue::Struct { ownership, .. } => *ownership,
            _ => Ownership::Trivial,
        }
    }

    pub fn as_llvm_basic_value(&self) -> BasicValueEnum<'a> {
        match self {
            GenValue::Integer(int_val) => BasicValueEnum::IntValue(*int_val),
            GenValue::Bool(int_val) => BasicValueEnum::IntValue(*int_val),
            GenValue::String { value: str_val, .. } => BasicValueEnum::PointerValue(*str_val),
            GenValue::Array { value: arr_val, .. } => BasicValueEnum::PointerValue(*arr_val),
            GenValue::Struct { value: struct_val, .. } => BasicValueEnum::StructValue(*struct_val),
            GenValue::Callable(callable_val) => BasicValueEnum::StructValue(*callable_val),
            GenValue::Void => panic!("Unexpected void value"),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn add_ref(&self, val: &GenValue<'ctxt>) -> Result<(), CodeGenError> {
        match val {
            GenValue::String { value: str_ptr, ownership: Ownership::Borrowed } => {
                self.builder.build_call(
                    self.runtime_functions.add_string_ref.into(),
                    &[(*str_ptr).into()],
                    "add_string_ref"
                )?;
            }
            GenValue::Array { value: array_ptr, ownership: Ownership::Borrowed, .. } => {
                self.builder.build_call(
                    self.runtime_functions.add_array_ref.into(),
                    &[(*array_ptr).into()],
                    "add_array_ref"
                )?;
            }
            GenValue::Struct { value: struct_value, struct_id, ownership: Ownership::Borrowed } => {
                if let Some(copy_fn) = self.struct_info[&struct_id].copy_fn {
                    self.builder.build_call(
                        copy_fn,
                        &[(*struct_value).into()],
                        "struct_copy"
                    )?;
                }
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn remove_ref(&self, val: GenValue<'ctxt>) -> Result<(), CodeGenError> {
        match val {
            GenValue::String { value: str_ptr, .. } => {
                self.builder.build_call(
                    self.runtime_functions.remove_string_ref.into(),
                    &[str_ptr.into()],
                    "remove_string_ref"
                )?;
            }
            GenValue::Array { value: array_ptr, .. } => {
                self.builder.build_call(
                    self.runtime_functions.remove_array_ref.into(),
                    &[array_ptr.into()],
                    "remove_array_ref"
                )?;
            }
            GenValue::Struct { value: struct_value, struct_id, .. } => {
                if let Some(drop_fn) = self.struct_info[&struct_id].drop_fn {
                    self.builder.build_call(
                        drop_fn,
                        &[struct_value.into()],
                        "struct_drop"
                    )?;
                }
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn remove_if_owned(&self, val: GenValue<'ctxt>) -> Result<(), CodeGenError> {
        if val.ownership() == Ownership::Owned {
            self.remove_ref(val)?;
        }
        Ok(())
    }

    pub fn llvm_basic_type(&self, sem_type: &SemanticType) -> BasicTypeEnum<'ctxt> {
        match sem_type.kind() {
            SemanticTypeKind::Integer => self.int_type().into(),
            SemanticTypeKind::Bool => self.bool_type().into(),
            SemanticTypeKind::String => self.ptr_type().into(),
            SemanticTypeKind::Array(_) => self.ptr_type().into(),
            SemanticTypeKind::NamedStruct(id, _) => self.struct_info[&id].struct_type.into(),
            SemanticTypeKind::Callable(_, _) => self.callable_struct_type.into(),
            _ => panic!("Incomplete type found in semantic IR"),
        }
    }
}