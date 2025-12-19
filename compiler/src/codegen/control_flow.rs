use inkwell::basic_block::BasicBlock;

use super::{CodeGen, CodeGenError, QLValue};
use crate::tokens::{ConditionalBranchNode, ExpressionNode, StatementNode};

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
        conditional_branches: Vec<ConditionalBranchNode>,
        else_branch: Option<Vec<StatementNode>>
    ) -> Result<bool, CodeGenError> {
        let initial_block = self.builder.get_insert_block().unwrap();
        let merge_block = self.append_block("merge_branches");

        let mut all_branches_terminate = true;
        let mut next_block: BasicBlock = merge_block;
        if let Some(else_body) = else_branch {
            let body_block = self.context.prepend_basic_block(merge_block, "else_body");
            next_block = body_block;

            let terminates = self.gen_block_stmts(body_block, else_body)?;
            all_branches_terminate = all_branches_terminate && terminates;
            self.branch_if(!terminates, body_block, merge_block)?;
        } else {
            all_branches_terminate = false;
        }

        for branch in conditional_branches.into_iter().rev() {
            let body_block = self.context.prepend_basic_block(next_block, "branch_body");
            let cond_block = self.context.prepend_basic_block(body_block, "branch_cond");

            self.builder.position_at_end(cond_block);
            let cond_val = branch.condition.gen_eval(self)?;
            if let QLValue::Bool(cond_llvm) = cond_val {
                self.builder.build_conditional_branch(cond_llvm, body_block, next_block)?;
            } else {
                return Err(CodeGenError::UnexpectedTypeError);
            }

            let terminates = self.gen_block_stmts(body_block, branch.body)?;
            all_branches_terminate = all_branches_terminate && terminates;
            self.branch_if(!terminates, body_block, merge_block)?;
            
            next_block = cond_block;
        }

        self.builder.position_at_end(initial_block);
        self.builder.build_unconditional_branch(next_block)?;

        if all_branches_terminate {
            self.builder.position_at_end(merge_block.get_previous_basic_block().unwrap());
            let _ = merge_block.remove_from_function();
            Ok(true)
        } else {
            self.builder.position_at_end(merge_block);
            Ok(false)
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