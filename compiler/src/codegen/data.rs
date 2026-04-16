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
    Iterator {
        value: PointerValue<'a>,
        elem_type: SemanticType,
        ownership: Ownership
    },
    Struct {
        value: StructValue<'a>,
        struct_id: u32,
        ownership: Ownership
    },
    Callable {
        value: PointerValue<'a>,
        ownership: Ownership
    },
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
            SemanticTypeKind::Iterator(elem_type) => GenValue::Iterator {
                value: llvm_value.into_pointer_value(),
                elem_type: elem_type,
                ownership: ownership
            },
            SemanticTypeKind::NamedStruct(struct_id, _) => GenValue::Struct {
                value: llvm_value.into_struct_value(),
                struct_id,
                ownership: ownership
            },
            SemanticTypeKind::Callable { .. } => GenValue::Callable {
                value: llvm_value.into_pointer_value(),
                ownership: ownership
            },
            SemanticTypeKind::Void => GenValue::Void,
            _ => panic!("Incomplete type found in semantic IR"),
        }
    }

    pub fn ownership(&self) -> Ownership {
        match self {
            GenValue::String { ownership, .. }
            | GenValue::Array { ownership, .. }
            | GenValue::Iterator { ownership, .. }
            | GenValue::Struct { ownership, .. }
            | GenValue::Callable { ownership, .. } => *ownership,
            _ => Ownership::Trivial,
        }
    }

    pub fn as_llvm_basic_value(&self) -> BasicValueEnum<'a> {
        match self {
            GenValue::Integer(int_val) => BasicValueEnum::IntValue(*int_val),
            GenValue::Bool(int_val) => BasicValueEnum::IntValue(*int_val),
            GenValue::String { value: str_val, .. } => BasicValueEnum::PointerValue(*str_val),
            GenValue::Array { value: arr_val, .. } => BasicValueEnum::PointerValue(*arr_val),
            GenValue::Iterator { value: iter_val, .. } => BasicValueEnum::PointerValue(*iter_val),
            GenValue::Struct { value: struct_val, .. } => BasicValueEnum::StructValue(*struct_val),
            GenValue::Callable { value: callable_val, .. } => BasicValueEnum::PointerValue(*callable_val),
            GenValue::Void => panic!("Unexpected void value"),
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn copy_value(&self, ptr: PointerValue, ptr_type: &SemanticType) -> Result<(), CodeGenError> {
        match ptr_type.kind() {
            SemanticTypeKind::String => {
                self.builder.build_call(
                    self.runtime.string_copy,
                    &[ptr.into()],
                    "copy_string"
                )?;
            }
            SemanticTypeKind::Array(_) => {
                self.builder.build_call(
                    self.runtime.array_copy,
                    &[ptr.into()],
                    "copy_array"
                )?;
            }
            SemanticTypeKind::Iterator(_) => {
                self.builder.build_call(
                    self.runtime.iterator_copy,
                    &[ptr.into()],
                    "copy_iterator"
                )?;
            }
            SemanticTypeKind::NamedStruct(struct_id, _) => {
                let sem_struct = &self.program.structs[&struct_id];
                let struct_info = &self.struct_info[&struct_id];
                if let Some(copy_fn) = struct_info.copy_fn {
                    self.builder.build_call(
                        copy_fn,
                        &[ptr.into()],
                        &format!("copy_struct_{}", sem_struct.name)
                    )?;
                }
            }
            SemanticTypeKind::Callable { .. } => {
                self.builder.build_call(
                    self.runtime.callable_copy,
                    &[ptr.into()],
                    "copy_callable"
                )?;
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn drop_value(&self, ptr: PointerValue, ptr_type: &SemanticType) -> Result<(), CodeGenError> {
        match ptr_type.kind() {
            SemanticTypeKind::String => {
                self.builder.build_call(
                    self.runtime.string_drop,
                    &[ptr.into()],
                    "drop_string"
                )?;
            }
            SemanticTypeKind::Array(_) => {
                self.builder.build_call(
                    self.runtime.array_drop,
                    &[ptr.into()],
                    "drop_array"
                )?;
            }
            SemanticTypeKind::Iterator(_) => {
                self.builder.build_call(
                    self.runtime.iterator_drop,
                    &[ptr.into()],
                    "drop_iterator"
                )?;
            }
            SemanticTypeKind::NamedStruct(struct_id, _) => {
                let sem_struct = &self.program.structs[&struct_id];
                let struct_info = &self.struct_info[&struct_id];
                if let Some(drop_fn) = struct_info.drop_fn {
                    self.builder.build_call(
                        drop_fn,
                        &[ptr.into()],
                        &format!("drop_struct_{}", sem_struct.name)
                    )?;
                }
            }
            SemanticTypeKind::Callable { .. } => {
                self.builder.build_call(
                    self.runtime.callable_drop,
                    &[ptr.into()],
                    "drop_callable"
                )?;
            }
            _ => { }
        }
        Ok(())
    }

    pub(super) fn put_on_stack(&self, val: &GenValue<'ctxt>, name: &str) -> Result<PointerValue<'ctxt>, CodeGenError> {
        let llvm_type = val.as_llvm_basic_value().get_type();
        let ptr = self.builder.build_alloca(llvm_type, name)?;
        self.builder.build_store(ptr, val.as_llvm_basic_value())?;
        Ok(ptr)
    }

    pub(super) fn remove_if_owned(&self, val: GenValue<'ctxt>, ptr_type: &SemanticType) -> Result<(), CodeGenError> {
        if val.ownership() == Ownership::Owned {
            let value_ptr = self.put_on_stack(&val, "owned_val_ptr")?;
            self.drop_value(value_ptr, ptr_type)?;
        }
        Ok(())
    }

    // Builds an alloca at the beginning of the function (to prevent dynamic stack growth)
    pub(super) fn build_alloca(&self, llvm_type: BasicTypeEnum<'ctxt>, name: &str) -> Result<PointerValue<'ctxt>, CodeGenError> {
        let cur_block = self.builder.get_insert_block().unwrap();
        let function = self.cur_fn.unwrap();
        let first_block = function.get_first_basic_block().unwrap();
        match first_block.get_first_instruction() {
            Some(instr) => self.builder.position_before(&instr),
            None => self.builder.position_at_end(first_block),
        }

        let alloca_ptr = self.builder.build_alloca(llvm_type, name)?;
        self.builder.position_at_end(cur_block);
        Ok(alloca_ptr)
    }

    pub fn llvm_basic_type(&self, sem_type: &SemanticType) -> BasicTypeEnum<'ctxt> {
        match sem_type.kind() {
            SemanticTypeKind::Integer => self.int_type().into(),
            SemanticTypeKind::Bool => self.bool_type().into(),
            SemanticTypeKind::String => self.ptr_type().into(),
            SemanticTypeKind::Array(_) => self.ptr_type().into(),
            SemanticTypeKind::Iterator(_) => self.ptr_type().into(),
            SemanticTypeKind::NamedStruct(id, _) => self.struct_info[&id].struct_type.into(),
            SemanticTypeKind::Callable { .. } => self.ptr_type().into(),
            _ => panic!("Incomplete type found in semantic IR"),
        }
    }
}