use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, ValueKind};

use super::{CodeGen, CodeGenError, QLValue};
use crate::tokens::StatementNode;

impl<'ctxt> CodeGen<'ctxt> {
    pub fn call(&self, fn_name: &str, args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
        let function = self.module.get_function(fn_name)
            .ok_or_else(|| CodeGenError::UndefinedVariableError(fn_name.to_string()))?;
        let arg_values: Vec<BasicMetadataValueEnum> = args
            .into_iter()
            .map(|v| BasicValueEnum::try_from(v).map(BasicMetadataValueEnum::from))
            .collect::<Result<Vec<BasicMetadataValueEnum>, CodeGenError>>()?;
        let call_site = self.builder.build_call(function, &arg_values, "call")?;
        match call_site.try_as_basic_value() {
            ValueKind::Basic(value) => Ok(value.try_into()?),
            ValueKind::Instruction(_) => Ok(QLValue::Void),
        }
    }

    pub fn gen_conditional(
        &mut self,
        conditional: QLValue<'ctxt>,
        then_stmts: &Vec<StatementNode>,
        else_stmts: &Vec<StatementNode>
    ) -> Result<(), CodeGenError> {
        if let QLValue::Bool(cond_bool) = conditional {
            let then_block = self.append_block("then");
            let else_block = self.append_block("else");
            let merge_block = self.append_block("merge");

            self.builder.build_conditional_branch(cond_bool, then_block, else_block)?;
            self.builder.position_at_end(then_block);
            self.gen_stmts(then_stmts)?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(else_block);
            self.gen_stmts(else_stmts)?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(merge_block);

            Ok(())
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}
