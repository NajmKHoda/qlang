use inkwell::{types::{BasicTypeEnum, StructType}, values::{BasicValueEnum}};

use crate::{codegen::QLValue, tokens::{ColumnValueNode, TypedQNameNode}};

use super::{CodeGen, CodeGenError, QLType};

struct QLTableColumn {
    name: String,
    ql_type: QLType,
}

impl From<&TypedQNameNode> for QLTableColumn {
    fn from(node: &TypedQNameNode) -> Self {
        QLTableColumn {
            name: node.name.clone(),
            ql_type: node.ql_type.clone(),
        }
    }
}

pub(super) struct QLTable<'ctxt> {
    struct_type: StructType<'ctxt>,
    fields: Vec<QLTableColumn>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_table(&mut self, name: &str, fields: &[TypedQNameNode]) -> Result<(), CodeGenError> {
        let field_types = fields.iter()
            .map(|f| self.try_get_nonvoid_type(&f.ql_type))
            .collect::<Result<Vec<BasicTypeEnum>, CodeGenError>>()?;
        
        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&field_types, false);

        let table = QLTable {
            struct_type,
            fields: fields.iter().map(|f| f.into()).collect(),
        };

        self.tables.insert(name.to_string(), table);

        Ok(())
    }

    pub fn gen_table_row(&self, table_name: &str, columns: &[ColumnValueNode]) -> Result<QLValue<'ctxt>, CodeGenError> {
        let table = self.tables.get(table_name)
            .ok_or_else(|| CodeGenError::UndefinedTableError(table_name.to_string()))?;

        let row_ptr = self.builder.build_alloca(table.struct_type, &format!("{}.row", table_name))?;
        for column in columns {
            let column_index = table.fields.iter()
                .position(|c| c.name == column.name)
                .ok_or_else(|| CodeGenError::UndefinedTableColumnError(column.name.clone(), table_name.to_string()))? as u32;

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

            self.builder.build_store(column_ptr, BasicValueEnum::try_from(column_value)?)?;
        }

        Ok(QLValue::TableRow(row_ptr, table_name.to_string()))
    }
}