use inkwell::basic_block::BasicBlock;

use super::{CodeGen, CodeGenError, QLValue};
use crate::tokens::{ExpressionNode, StatementNode};

impl<'ctxt> CodeGen<'ctxt> {
    fn branch_if(
        &mut self,
        predicate: bool,
        source_block: BasicBlock<'ctxt>,
        destination_block: BasicBlock<'ctxt>
    ) -> Result<(), CodeGenError> {
        self.builder.position_at_end(source_block);
        if predicate {
            self.builder.build_unconditional_branch(destination_block)?;
        }
        Ok(())
    }

    pub fn gen_conditional(
        &mut self,
        conditional: QLValue<'ctxt>,
        then_stmts: Vec<StatementNode>,
        else_stmts: Vec<StatementNode>
    ) -> Result<bool, CodeGenError> {
        if let QLValue::Bool(cond_bool) = conditional {
            let then_block = self.append_block("then");
            let else_block = self.append_block("else");
            self.builder.build_conditional_branch(cond_bool, then_block, else_block)?;

            let then_terminates = self.gen_block_stmts(then_block, then_stmts)?;
            let else_terminates = self.gen_block_stmts(else_block, else_stmts)?;

            if then_terminates && else_terminates {
                // No need to create a merge block if both terminate
                return Ok(true);
            }

            let merge_block = self.append_block("merge");
            self.branch_if(!then_terminates, then_block, merge_block)?;
            self.branch_if(!else_terminates, else_block, merge_block)?;
            self.builder.position_at_end(merge_block);

            Ok(false)
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_loop(
        &mut self,
        condition_expr: Box<ExpressionNode>,
        body_stmts: Vec<StatementNode>
    ) -> Result<(), CodeGenError> {
        let loop_cond_block = self.append_block("loop_cond");
        let loop_body_block = self.append_block("loop_body");
        let after_loop_block = self.append_block("after_loop");

        self.builder.build_unconditional_branch(loop_cond_block)?;

        self.builder.position_at_end(loop_cond_block);
        let condition = condition_expr.gen_eval(self)?;
        if let QLValue::Bool(cond_bool) = condition {
            self.builder.build_conditional_branch(cond_bool, loop_body_block, after_loop_block)?;
        } else {
            return Err(CodeGenError::UnexpectedTypeError);
        }

        let body_terminates = self.gen_block_stmts(loop_body_block, body_stmts)?;
        self.branch_if(!body_terminates, loop_body_block, loop_cond_block)?;

        self.builder.position_at_end(after_loop_block);

        Ok(())
    }
}