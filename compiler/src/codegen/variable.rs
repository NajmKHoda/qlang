use crate::semantics::Ownership;
use super::{CodeGen, CodeGenError, data::GenValue};

impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn load_var(&self, variable_id: u32) -> Result<GenValue<'ctxt>, CodeGenError> {
        let variable_ptr = self.llvm_variables[&variable_id];
        let var_type = &self.program.variables[&variable_id].sem_type;
        let llvm_type = self.llvm_basic_type(var_type);
        let loaded_value = self.builder.build_load(llvm_type, variable_ptr, "load")?;
        Ok(GenValue::new(var_type, loaded_value, Ownership::Borrowed))
    }

    pub(super) fn define_var(&mut self, variable_id: u32, value: GenValue<'ctxt>) -> Result<(), CodeGenError> {
        let variable = &self.program.variables[&variable_id];
        let llvm_type = self.llvm_basic_type(&variable.sem_type);
        let variable_ptr = self.builder.build_alloca(llvm_type, &variable.name)?;

        self.builder.build_store(variable_ptr, value.as_llvm_basic_value())?;
        self.llvm_variables.insert(variable.id, variable_ptr); 
        if value.ownership() == Ownership::Borrowed {
            self.copy_value(variable_ptr, &variable.sem_type)?;
        }

        Ok(())
    }

    pub(super) fn store_var(&self, variable_id: u32, value: GenValue<'ctxt>) -> Result<(), CodeGenError> {
        let variable_ptr = self.llvm_variables[&variable_id];
        let var_type = &self.program.variables[&variable_id].sem_type;
        if var_type.can_be_owned() {
            self.drop_value(variable_ptr, var_type)?;
        }

        self.builder.build_store(variable_ptr, value.as_llvm_basic_value())?;
        if value.ownership() == Ownership::Borrowed {
            self.copy_value(variable_ptr, var_type)?;
        }

        Ok(())
    }

    pub(super) fn drop_var(&self, variable_id: u32) -> Result<(), CodeGenError> {
        let variable = &self.program.variables[&variable_id];
        let variable_ptr = self.llvm_variables[&variable_id];
        self.drop_value(variable_ptr, &variable.sem_type)?;
        Ok(())
    }
}