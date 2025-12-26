use std::collections::HashSet;

use inkwell::{types::{BasicTypeEnum, StructType}, values::{BasicValueEnum}};

use crate::{codegen::QLValue, tokens::{ColumnValueNode, TypedQNameNode}};

use super::{CodeGen, CodeGenError, QLType};

pub(super) struct QLTableColumn {
    pub(super) name: String,
    pub(super) ql_type: QLType,
}

impl From<&TypedQNameNode> for QLTableColumn {
    fn from(node: &TypedQNameNode) -> Self {
        QLTableColumn {
            name: node.name.clone(),
            ql_type: node.ql_type.clone(),
        }
    }
}
    
pub(super) struct QLTable<'a> {
    pub(super) name: String,
    pub(super) struct_type: StructType<'a>,
    pub(super) fields: Vec<QLTableColumn>,
}

impl<'a> QLTable<'a> {
    pub fn get_column_index(&self, column_name: &str) -> Result<u32, CodeGenError> {
        self.fields.iter().position(|c| c.name == column_name)
            .map(|idx| idx as u32)
            .ok_or_else(|| CodeGenError::UndefinedTableColumnError(column_name.to_string(), self.name.clone()))
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_table(&mut self, name: &str, fields: &[TypedQNameNode]) -> Result<(), CodeGenError> {
        let field_types = fields.iter()
            .map(|f| self.try_get_nonvoid_type(&f.ql_type))
            .collect::<Result<Vec<BasicTypeEnum>, CodeGenError>>()?;
        
        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&field_types, false);

        let table = QLTable {
            name: name.to_string(),
            struct_type,
            fields: fields.iter().map(|f| f.into()).collect(),
        };

        self.tables.insert(name.to_string(), table);

        Ok(())
    }

    pub fn gen_table_row(&self, table_name: &str, columns: &[ColumnValueNode]) -> Result<QLValue<'ctxt>, CodeGenError> {
        let table = self.tables.get(table_name)
            .ok_or_else(|| CodeGenError::UndefinedTableError(table_name.to_string()))?;

        let row_ptr = self.builder.build_alloca(table.struct_type, &format!("{}.row.store", table_name))?;
        let mut remaining_columns: HashSet<_> = (0..table.fields.len() as u32).collect();
        for column in columns {
            let column_index = table.get_column_index(&column.name)?;
            if !remaining_columns.contains(&column_index) {
                return Err(CodeGenError::DuplicateColumnAssignmentError(column.name.clone(), table_name.to_string()));
            }

            let column_ptr = self.builder.build_struct_gep(
                table.struct_type,
                row_ptr, 
                column_index,
                &format!("{}.{}", table_name, column.name)
            )?;

            let column_value = column.value.gen_eval(&self)?;
            if column_value.get_type() != table.fields[column_index as usize].ql_type {
                return Err(CodeGenError::UnexpectedTypeError);
            }

            self.add_ref(&column_value)?;
            self.builder.build_store(column_ptr, BasicValueEnum::try_from(column_value)?)?;
            remaining_columns.remove(&column_index);
        }

        if remaining_columns.len() != 0 {
            return Err(CodeGenError::MissingColumnAssignmentError(table_name.to_string()));
        }

        let struct_val = self.builder.build_load(
            table.struct_type,
            row_ptr,
            &format!("{}.row.load", table_name)
        )?.into_struct_value();
        Ok(QLValue::TableRow(struct_val, table_name.to_string(), false))
    }

    pub fn get_column_value(&self, table_row: QLValue<'ctxt>, column_name: &str) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let QLValue::TableRow(struct_val, table_name, _) = table_row {
            let table = self.tables.get(&table_name)
                .ok_or_else(|| CodeGenError::UndefinedTableError(table_name.clone()))?;

            let column_index = table.get_column_index(column_name)?;
            let column_type = &table.fields[column_index as usize].ql_type;
            let loaded_val = self.builder.build_extract_value(
                struct_val,
                column_index,
                &format!("{}.{}", table_name, column_name)
            )?;

            Ok(column_type.to_value(loaded_val, true))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}