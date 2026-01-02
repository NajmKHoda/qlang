use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, fmt::Display, rc::Rc};

use super::*;

#[derive(Clone)]
pub(super) enum SemanticTypeKind {
    Any,
    Integer,
    Bool,
    String,
    Array(SemanticType),
    NamedStruct(Rc<SemanticStruct>),
    AnonymousStruct(HashMap<String, SemanticType>),
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
            (SemanticTypeKind::NamedStruct(struct_a), SemanticTypeKind::NamedStruct(struct_b)) => Rc::ptr_eq(struct_a, struct_b),
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
}

impl Display for SemanticTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticTypeKind::Any => write!(f, "any"),
            SemanticTypeKind::Integer => write!(f, "int"),
            SemanticTypeKind::Bool => write!(f, "bool"),
            SemanticTypeKind::String => write!(f, "str"),
            SemanticTypeKind::Array(elem_type) => write!(f, "{}[]", elem_type),
            SemanticTypeKind::NamedStruct(named_struct) => write!(f, "{}", &named_struct.name),
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
            SemanticTypeKind::Void => write!(f, "void"),
        }
    }
}

#[derive(Clone)]
pub(super) struct SemanticType(Rc<RefCell<SemanticTypeKind>>);

impl SemanticType {
    fn borrow(&self) -> Ref<SemanticTypeKind> {
        self.0.borrow()
    }

    fn borrow_mut(&self) -> RefMut<SemanticTypeKind> {
        self.0.borrow_mut()
    }

    pub(super) fn new(kind: SemanticTypeKind) -> Self {
        SemanticType(Rc::new(RefCell::new(kind)))
    }

    pub(super) fn try_unify(a: &SemanticType, b: &SemanticType) -> bool {
        return a.try_downcast(b) || b.try_downcast(a);
    }

    pub(super) fn kind(&self) -> SemanticTypeKind {
        self.0.borrow().clone()
    }

    pub(super) fn is_concrete(&self) -> bool {
        let borrowed = &*self.borrow();
        match borrowed {
            SemanticTypeKind::Any => false,
            SemanticTypeKind::Array(elem_type) => elem_type.is_concrete(),
            SemanticTypeKind::AnonymousStruct(_) => false,
            _ => true
        }
    }

    pub(super) fn try_downcast(&self, target: &SemanticType) -> bool {
        let target_borrowed = target.borrow();
        let mut sem_type_borrowed = self.borrow_mut();
        match (&*target_borrowed, &mut *sem_type_borrowed) {
            (other, any @ SemanticTypeKind::Any) => {
                *any = other.clone();
                true
            },
            (SemanticTypeKind::Integer, SemanticTypeKind::Integer) => true,
            (SemanticTypeKind::Bool, SemanticTypeKind::Bool) => true,
            (SemanticTypeKind::String, SemanticTypeKind::String) => true,
            (SemanticTypeKind::Array(elem_a), SemanticTypeKind::Array(elem_b)) => {
                elem_b.try_downcast(elem_a)
            }
            (SemanticTypeKind::NamedStruct(struct_a), SemanticTypeKind::NamedStruct(struct_b)) => Rc::ptr_eq(struct_a, struct_b),
            (SemanticTypeKind::NamedStruct(named_struct), SemanticTypeKind::AnonymousStruct(fields)) => {
                let target_fields = &named_struct.fields;
                if Self::try_downcast_struct(target_fields, fields) {
                    *sem_type_borrowed = SemanticTypeKind::NamedStruct(named_struct.clone());
                    true
                } else {
                    false
                }
            }
            (SemanticTypeKind::AnonymousStruct(target_fields), SemanticTypeKind::AnonymousStruct(struct_fields)) => {
                Self::try_downcast_struct(target_fields, struct_fields)
            }
            _ => false
        }
    }

    pub(super) fn try_downcast_struct(
        target_fields: &HashMap<String, SemanticType>,
        struct_fields: &mut HashMap<String, SemanticType>
    ) -> bool {
        if target_fields.len() != struct_fields.len() {
            return false;
        }
        for (field_name, field_type) in struct_fields {
            match target_fields.get(field_name) {
                Some(col_type) => {
                    if !field_type.try_downcast(col_type) {
                        return false;
                    }
                },
                None => return false
            }
        }
        true
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
                if let Some(named_struct) = self.structs.get(struct_name) {
                    Ok(SemanticType::new(SemanticTypeKind::NamedStruct(named_struct.clone())))
                } else {
                    Err(SemanticError::UndefinedStruct { name: struct_name.to_string() })
                }
            }
        }
    }
}