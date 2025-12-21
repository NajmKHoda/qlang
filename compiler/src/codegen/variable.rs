use std::collections::HashMap;

use inkwell::values::{BasicValueEnum, PointerValue};

use super::{CodeGen, CodeGenError, QLValue, QLType};

pub(super) struct QLVariable<'a> {
    pub(super) ql_type: QLType,
    pointer: PointerValue<'a>,
}

pub(super) enum QLScopeType {
    FunctionScope,
    ConditionalScope,
    LoopScope(Option<String>),
}
pub(super) struct QLScope<'a> {
    pub(super) vars: HashMap<String, QLVariable<'a>>,
    pub(super) scope_type: QLScopeType,
}

impl<'a> QLScope<'a> {
    pub fn new(scope_type: QLScopeType) -> Self {
        QLScope {
            vars: HashMap::new(),
            scope_type,
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    fn get_var<'a>(&'a self, name: &str) -> Option<&'a QLVariable<'ctxt>> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var);
            }
        }
        None
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
        let cur_scope = self.scopes.last_mut().unwrap();
        if cur_scope.vars.contains_key(name) {
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
        cur_scope.vars.insert(name.to_string(), var);
        
        Ok(())
    }

    pub fn store_var(&self, name: &str, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if let Some(variable) = self.get_var(name) {
            if variable.ql_type != value.get_type() {
                return Err(CodeGenError::UnexpectedTypeError);
            }

            let prev_value = self.load_var(name)?;
            self.remove_ref(prev_value)?;
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