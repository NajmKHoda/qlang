use super::{CodeGen, CodeGenError, QLValue};
use crate::tokens::{ExpressionNode, StatementNode};

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_conditional(
        &mut self,
        conditional: QLValue<'ctxt>,
        then_stmts: Vec<StatementNode>,
        else_stmts: Vec<StatementNode>
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

        self.builder.position_at_end(loop_body_block);
        self.gen_stmts(body_stmts)?;
        self.builder.build_unconditional_branch(loop_cond_block)?;

        self.builder.position_at_end(after_loop_block);

        Ok(())
    }
}
