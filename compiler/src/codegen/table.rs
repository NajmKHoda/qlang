use inkwell::{builder::BuilderError, values::GlobalValue};

use crate::semantics::SemanticTable;
use super::{CodeGen, CodeGenError};
    
pub(super) struct GenTableInfo<'a> {
    pub(super) name_str: GlobalValue<'a>,
    pub(super) column_name_strs: Vec<GlobalValue<'a>>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_table(&mut self, table: &SemanticTable) -> Result<(), CodeGenError> {
        let table_struct = &self.program.structs[&table.struct_id];
        self.gen_struct(table_struct)?;

        let name_str = self.builder.build_global_string_ptr(&table.name, &format!("{}_name", table.name))?;
        let column_name_strs = table_struct.field_order
            .iter().enumerate()
            .map(|(i, field_name)| {
                self.builder.build_global_string_ptr(
                    field_name,
                    &format!("{}_col_{}", table.name, i)
                )
            }).collect::<Result<Vec<GlobalValue>, BuilderError>>()?;
        
        let gen_table_info = GenTableInfo {
            name_str,
            column_name_strs,
        };

        self.table_info.insert(table.id, gen_table_info);

        Ok(())
    }
}