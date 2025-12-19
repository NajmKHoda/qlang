use inkwell::basic_block::BasicBlock;

use super::{CodeGen, CodeGenError, QLValue};
use crate::tokens::{ExpressionNode, StatementNode};

pub(super) struct QLLoop<'a> {
    label: Option<String>,
    cond_block: BasicBlock<'a>,
    after_block: BasicBlock<'a>
}

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

    fn find_loop(&self, label_op: Option<String>) -> Result<&QLLoop<'ctxt>, CodeGenError> {
        match label_op {
            Some(label) => self.loops.iter()
                .find(|lp| lp.label.as_ref() == Some(&label))
                .ok_or(CodeGenError::UndefinedLoopLabelError(label)),
            None => self.loops.last().ok_or(CodeGenError::BadLoopControlError)
        }
    }

    pub fn gen_conditional(
        &mut self,
        conditional: QLValue<'ctxt>,
        then_stmts: Vec<StatementNode>,
        else_stmts: Vec<StatementNode>
    ) -> Result<bool, CodeGenError> {
        if let QLValue::Bool(cond_bool) = conditional {
            let then_entry_block = self.append_block("then_entry");
            let else_entry_block = self.append_block("else_entry");
            self.builder.build_conditional_branch(cond_bool, then_entry_block, else_entry_block)?;

            let then_terminates = self.gen_block_stmts(then_entry_block, then_stmts)?;
            let then_tail_block = self.builder.get_insert_block().unwrap();

            let else_terminates = self.gen_block_stmts(else_entry_block, else_stmts)?;
            let else_tail_block = self.builder.get_insert_block().unwrap();

            if then_terminates && else_terminates {
                // No need to create a merge block if both terminate
                return Ok(true);
            }

            let merge_block = self.append_block("merge");
            self.branch_if(!then_terminates, then_tail_block, merge_block)?;
            self.branch_if(!else_terminates, else_tail_block, merge_block)?;
            self.builder.position_at_end(merge_block);

            Ok(false)
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_loop(
        &mut self,
        condition_expr: Box<ExpressionNode>,
        body_stmts: Vec<StatementNode>,
        loop_label: Option<String>
    ) -> Result<(), CodeGenError> {
        let loop_cond_block = self.append_block("loop_cond");
        let loop_body_entry_block = self.append_block("loop_body_entry");
        let after_loop_block = self.append_block("after_loop");

        self.builder.build_unconditional_branch(loop_cond_block)?;

        self.builder.position_at_end(loop_cond_block);
        let condition = condition_expr.gen_eval(self)?;
        if let QLValue::Bool(cond_bool) = condition {
            self.builder.build_conditional_branch(cond_bool, loop_body_entry_block, after_loop_block)?;
        } else {
            return Err(CodeGenError::UnexpectedTypeError);
        }

        self.loops.push(QLLoop {
            label: loop_label,
            cond_block: loop_cond_block,
            after_block: after_loop_block
        });

        let body_terminates = self.gen_block_stmts(loop_body_entry_block, body_stmts)?;
        let cur_block = self.builder.get_insert_block().unwrap();
        self.branch_if(!body_terminates, cur_block, loop_cond_block)?;

        self.loops.pop();
        self.builder.position_at_end(after_loop_block);

        Ok(())
    }

    pub fn gen_break(&mut self, label: Option<String>) -> Result<(), CodeGenError> {
        let loop_info = self.find_loop(label)?;
        self.builder.build_unconditional_branch(loop_info.after_block)?;
        Ok(())
    }

    pub fn gen_continue(&mut self, label: Option<String>) -> Result<(), CodeGenError> {
        let loop_info = self.find_loop(label)?;
        self.builder.build_unconditional_branch(loop_info.cond_block)?;
        Ok(())
    }
}