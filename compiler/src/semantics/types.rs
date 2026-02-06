use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, fmt::Display, rc::Rc};

use super::*;

#[derive(Clone)]
pub enum SemanticTypeKind {
    Any,
    Integer,
    Bool,
    String,
    Array(SemanticType),
    NamedStruct(u32, String),
    AnonymousStruct(HashMap<String, SemanticType>),
    Callable(Vec<SemanticType>, SemanticType),
    Void
}

impl PartialEq for SemanticTypeKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SemanticTypeKind::Any, SemanticTypeKind::Any) => true,
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => true,
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool) => true,
            (SemanticTypeKind::String, SemanticTypeKind::String) => true,
            (SemanticTypeKind::Array(elem_a), SemanticTypeKind::Array(elem_b)) => elem_a == elem_b,
            (SemanticTypeKind::NamedStruct(id_a, _), SemanticTypeKind::NamedStruct(id_b, _)) => id_a == id_b,
            (SemanticTypeKind::AnonymousStruct(fields_a), SemanticTypeKind::AnonymousStruct(fields_b)) => fields_a == fields_b,
            (SemanticTypeKind::Void, SemanticTypeKind::Void) => true,
            _ => false
        }
    }
}

impl SemanticTypeKind {
    fn is_concrete(&self) -> bool {
        match self {
            SemanticTypeKind::Any => false,
            SemanticTypeKind::Array(elem_type) => elem_type.is_concrete(),
            SemanticTypeKind::AnonymousStruct(_) => false,
            _ => true
        }
    }

    fn can_be_owned(&self) -> bool {
        match self {
            SemanticTypeKind::String => true,
            SemanticTypeKind::Array(_) => true,
            SemanticTypeKind::NamedStruct(_, _) => true,
            _ => false
        }
    }
}

impl Display for SemanticTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticTypeKind::Any => write!(f, "any"),
            SemanticTypeKind::Integer => write!(f, "int"),
            SemanticTypeKind::Bool => write!(f, "bool"),
            SemanticTypeKind::String => write!(f, "str"),
            SemanticTypeKind::Array(elem_type) => write!(f, "{}[]", elem_type),
            SemanticTypeKind::NamedStruct(_, name) => write!(f, "{}", name),
            SemanticTypeKind::AnonymousStruct(fields) => {
                write!(f, "{{")?;
                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    write!(f, "{field_name}: {field_type}")?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            SemanticTypeKind::Callable(param_types, ret_type) => {
                write!(f, "(")?;
                for (i, param_type) in param_types.iter().enumerate() {
                    write!(f, "{}", param_type)?;
                    if i < param_types.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") -> {}", ret_type)
            }
            SemanticTypeKind::Void => write!(f, "void"),
        }
    }
}

#[derive(Clone)]
pub struct SemanticType(Rc<RefCell<SemanticTypeKind>>);

impl SemanticType {
    fn borrow<'a>(&'a self) -> Ref<'a, SemanticTypeKind> {
        self.0.borrow()
    }

    fn borrow_mut<'a>(&'a self) -> RefMut<'a, SemanticTypeKind> {
        self.0.borrow_mut()
    }

    pub fn new(kind: SemanticTypeKind) -> Self {
        SemanticType(Rc::new(RefCell::new(kind)))
    }

    pub(super) fn is_concrete(&self) -> bool {
        (*self.borrow()).is_concrete()
    }

    pub fn can_be_owned(&self) -> bool {
        (*self.borrow()).can_be_owned()
    }

    pub fn kind(&self) -> SemanticTypeKind {
        self.0.borrow().clone()
    }
}

impl Display for SemanticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let borrowed = &*self.borrow();
        write!(f, "{}", borrowed)
    }
}

impl PartialEq for SemanticType {
    fn eq(&self, other: &Self) -> bool {
        let self_borrowed = &*self.borrow();
        let other_borrowed = &*other.borrow();
        self_borrowed == other_borrowed
    }
}

impl PartialEq<SemanticTypeKind> for SemanticType {
    fn eq(&self, other: &SemanticTypeKind) -> bool {
        let self_borrowed = &*self.borrow();
        self_borrowed == other
    }
}

impl SemanticGen {
    pub fn try_get_semantic_type(&self, type_node: &TypeNode) -> Result<SemanticType, SemanticError> {
        match type_node {
            TypeNode::Integer => Ok(SemanticType::new(SemanticTypeKind::Integer)),
            TypeNode::Bool => Ok(SemanticType::new(SemanticTypeKind::Bool)),
            TypeNode::String => Ok(SemanticType::new(SemanticTypeKind::String)),
            TypeNode::Array(elem_type_node) => {
                let elem_type = self.try_get_semantic_type(elem_type_node)?;
                Ok(SemanticType::new(SemanticTypeKind::Array(elem_type)))
            },
            TypeNode::Struct(struct_name) => {
                if let Some(named_struct) = self.structs.get_by_name(struct_name) {
                    Ok(SemanticType::new(SemanticTypeKind::NamedStruct(named_struct.id, struct_name.clone())))
                } else {
                    Err(SemanticError::UndefinedStruct { name: struct_name.to_string() })
                }
            },
            TypeNode::Void => Ok(SemanticType::new(SemanticTypeKind::Void)),
        }
    }

    pub(super) fn try_unify(&self, a: &SemanticType, b: &SemanticType) -> bool {
        return self.try_downcast(b, a) || self.try_downcast(a, b);
    }

    pub(super) fn try_downcast(&self, target: &SemanticType, sem_type: &SemanticType) -> bool {
        let target_kind = target.kind();
        let self_kind = sem_type.kind();
        match (target_kind, self_kind) {
            (other, SemanticTypeKind::Any) => {
                *(sem_type.borrow_mut()) = other;
                true
            },
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => true,
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool) => true,
            (SemanticTypeKind::String, SemanticTypeKind::String) => true,
            (SemanticTypeKind::Array(elem_a), SemanticTypeKind::Array(elem_b)) => self.try_downcast(&elem_a, &elem_b),
            (SemanticTypeKind::NamedStruct(struct_a, _), SemanticTypeKind::NamedStruct(struct_b, _))
                => struct_a == struct_b,
            (SemanticTypeKind::NamedStruct(struct_id, struct_name), SemanticTypeKind::AnonymousStruct(ref mut fields)) => {
                let target_fields = &self.structs[struct_id].fields;
                if self.try_downcast_struct(target_fields, fields) {
                    *(sem_type.borrow_mut()) = SemanticTypeKind::NamedStruct(struct_id, struct_name);
                    true
                } else {
                    false
                }
            }
            (SemanticTypeKind::AnonymousStruct(ref mut target_fields),
            SemanticTypeKind::AnonymousStruct(ref mut struct_fields)) => {
                self.try_downcast_struct(target_fields, struct_fields)
            }
            _ => false
        }
    }

    pub(super) fn try_downcast_struct(
        &self,
        target_fields: &HashMap<String, SemanticType>,
        struct_fields: &mut HashMap<String, SemanticType>
    ) -> bool {
        if target_fields.len() != struct_fields.len() {
            return false;
        }
        for (field_name, field_type) in struct_fields {
            match target_fields.get(field_name) {
                Some(target_type) => {
                    if !self.try_downcast(target_type, field_type) {
                        return false;
                    }
                },
                None => return false
            }
        }
        true
    }
}