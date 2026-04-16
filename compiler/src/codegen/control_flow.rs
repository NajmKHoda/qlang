use inkwell::{basic_block::BasicBlock, values::{AnyValue, BasicValue, IntValue}};

use super::{CodeGen, CodeGenError};
use crate::{semantics::{SemanticBlock, SemanticConditionalBranch, SemanticExpression}};

pub(super) struct GenLoopInfo<'a> {
    cond_block: BasicBlock<'a>,
    after_block: BasicBlock<'a>
}

#[derive(Clone, Copy)]
pub(super) struct GenTransactionInfo<'a> {
    pub(super) rollback_block: BasicBlock<'a>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_conditional(
        &mut self,
        conditional_branches: &[SemanticConditionalBranch],
        else_branch: &Option<SemanticBlock>
    ) -> Result<(), CodeGenError> {
        let cur_fn = self.cur_fn.unwrap();
        let initial_block = self.builder.get_insert_block().unwrap();

        struct BranchGenInfo<'a> {
            cond_value: IntValue<'a>,
            cond_block: BasicBlock<'a>,
            body_block: BasicBlock<'a>,
            body_terminates: bool,
        }

        // First pass: generate blocks
        let mut blocks: Vec<BranchGenInfo> = vec![];
        for (i, branch) in conditional_branches.iter().enumerate() {
            let cond_block = self.context.append_basic_block(cur_fn, &format!("branch{}_cond", i+1));
            self.builder.position_at_end(cond_block);
            let cond_value = self.gen_eval(&branch.condition)?.as_llvm_basic_value().into_int_value();

            let body_block = self.context.append_basic_block(cur_fn, &format!("branch{}_body", i+1));
            self.builder.position_at_end(body_block);
            self.gen_block(&branch.body)?;

            blocks.push(BranchGenInfo {
                cond_value,
                cond_block,
                body_block,
                body_terminates: branch.body.terminates,
            });
        }
        if let Some(else_block) = else_branch {
            let else_jump_block = self.context.append_basic_block(cur_fn, "else_jump");
            let else_body_block = self.context.append_basic_block(cur_fn, "else_body");
            self.builder.position_at_end(else_body_block);
            self.gen_block(else_block)?;

            blocks.push(BranchGenInfo {
                cond_value: self.context.bool_type().const_int(1, false),
                cond_block: else_jump_block,
                body_block: else_body_block,
                body_terminates: else_block.terminates,
            });
        }

        // Second pass: link blocks together
        for window in blocks.windows(2) {
            let BranchGenInfo { cond_value, cond_block, body_block, .. } = window[0];
            let BranchGenInfo { cond_block: next_cond_block, .. } = window[1];
            self.builder.position_at_end(cond_block);
            self.builder.build_conditional_branch(cond_value, body_block, next_cond_block)?;
        }

        // Link initial block to first condition block
        let BranchGenInfo { cond_block: first_cond_block, .. } = blocks.first().unwrap();
        self.builder.position_at_end(initial_block);
        self.builder.build_unconditional_branch(*first_cond_block)?;

        let all_branches_terminate = if let Some(else_block) = else_branch {
            else_block.terminates
            && conditional_branches.iter().all(|branch| branch.body.terminates)
        } else {
            false
        };
        
        let BranchGenInfo {
            cond_value: last_cond_value,
            cond_block: last_cond_block,
            body_block: last_body_block, ..
        } = blocks.last().unwrap();
        
        // If not all branches terminate, create a merge block
        if !all_branches_terminate {
            let merge_block = self.context.append_basic_block(cur_fn, "merge_branches");
            for BranchGenInfo { body_block, body_terminates, .. } in &blocks {
                if !*body_terminates {
                    self.builder.position_at_end(*body_block);
                    self.builder.build_unconditional_branch(merge_block)?;
                }
            }

            // Link last condition block to merge block
            self.builder.position_at_end(*last_cond_block);
            self.builder.build_conditional_branch(*last_cond_value, *last_body_block, merge_block)?;
            self.builder.position_at_end(merge_block);
        } else {
            self.builder.position_at_end(*last_cond_block);
            self.builder.build_unconditional_branch(*last_body_block)?;
        }

        Ok(())
    }

    pub fn gen_loop(
        &mut self,
        condition_expr: &SemanticExpression,
        body: &SemanticBlock,
        id: u32,
    ) -> Result<(), CodeGenError> {
        let cur_fn = self.cur_fn.unwrap();
        let cond_block = self.context.append_basic_block(cur_fn, "loop_cond");
        let entry_block = self.context.append_basic_block(cur_fn, "loop_body_entry");
        let after_block = self.context.append_basic_block(cur_fn, "after_loop");
        self.loop_info.insert(id, GenLoopInfo { cond_block, after_block });

        // Build loop conditional branch
        self.builder.build_unconditional_branch(cond_block)?;
        self.builder.position_at_end(cond_block);
        let condition = self.gen_eval(condition_expr)?.as_llvm_basic_value().into_int_value();
        self.builder.build_conditional_branch(condition, entry_block, after_block)?;

        // Build loop body
        self.builder.position_at_end(entry_block);
        self.gen_block(body)?;
        if !body.terminates {
            self.builder.build_unconditional_branch(cond_block)?;
        }

        let last_body_block = cur_fn.get_last_basic_block().unwrap();
        let _ = after_block.move_after(last_body_block);
        self.builder.position_at_end(after_block);
        Ok(())
    }

    pub fn gen_block(&mut self, block: &SemanticBlock) -> Result<(), CodeGenError> {
        for stmt in &block.statements {
            self.gen_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn gen_transaction(
        &mut self,
        body: &SemanticBlock,
        rollback_body: &SemanticBlock,
        id: u32,
    ) -> Result<(), CodeGenError> {
        let cur_fn = self.cur_fn.unwrap();

        let body_block = self.context.append_basic_block(cur_fn, "tx_body");
        let rollback_block = self.context.append_basic_block(cur_fn, "tx_rollback");
        let release_block = self.context.append_basic_block(cur_fn, "tx_release");
        let after_block = self.context.append_basic_block(cur_fn, "tx_after");

        let savepoint_ok = self.builder.build_call(
            self.runtime.db_savepoint,
            &[self.int_type().const_int(id as u64, false).into()],
            "tx_savepoint",
        )?.as_any_value_enum().into_int_value();
        self.builder.build_conditional_branch(savepoint_ok, body_block, rollback_block)?;

        self.builder.position_at_end(body_block);
        self.transaction_stack.push(GenTransactionInfo { rollback_block });
        self.gen_block(body)?;
        self.transaction_stack.pop();
        if !body.terminates {
            self.builder.build_unconditional_branch(release_block)?;
        }

        self.builder.position_at_end(rollback_block);
        self.builder.build_call(
            self.runtime.db_rollback_to_savepoint,
            &[self.int_type().const_int(id as u64, false).into()],
            "tx_rollback_to_sp",
        )?;
        self.gen_block(rollback_body)?;
        if !rollback_body.terminates {
            self.builder.build_unconditional_branch(release_block)?;
        }

        self.builder.position_at_end(release_block);
        self.builder.build_call(
            self.runtime.db_release_savepoint,
            &[self.int_type().const_int(id as u64, false).into()],
            "tx_release_sp",
        )?;
        self.builder.build_unconditional_branch(after_block)?;

        self.builder.position_at_end(after_block);
        Ok(())
    }

    pub fn gen_return(&mut self, value: &Option<u32>) -> Result<(), CodeGenError> {
        if self.cur_executable_is_failable() {
            let return_type = self.cur_executable_return_type();
            let result_ty = self.llvm_result_type(&return_type);
            let result_alloca = self.build_alloca(result_ty.into(), "failable_ok_result")?;
            let is_error_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 0, "ok_result_is_error_ptr")?;
            self.builder.build_store(is_error_ptr, self.bool_type().const_int(0, false))?;

            if let Some(var_id) = value {
                if return_type != crate::semantics::SemanticTypeKind::Void {
                    let ok_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 1, "ok_result_value_ptr")?;
                    let ret_val = self.load_var(*var_id)?;
                    self.builder.build_store(ok_ptr, ret_val.as_llvm_basic_value())?;
                }
            }

            let result_val = self.builder.build_load(result_ty, result_alloca, "failable_ok_result_val")?.into_struct_value();
            self.builder.build_return(Some(&result_val))?;
        } else {
            let return_val: Option<&dyn BasicValue> = if let Some(var_id) = value {
                let ret_val = self.load_var(*var_id)?;
                Some(&ret_val.as_llvm_basic_value())
            } else {
                None
            };
            self.builder.build_return(return_val)?;
        }
		Ok(())
	}

    pub fn gen_break(&mut self, loop_id: u32) -> Result<(), CodeGenError> {
        let GenLoopInfo { after_block, .. } = self.loop_info[&loop_id];
        self.builder.build_unconditional_branch(after_block)?;
        Ok(())
    }

    pub fn gen_continue(&mut self, loop_id: u32) -> Result<(), CodeGenError> {
        let GenLoopInfo { cond_block, .. } = self.loop_info[&loop_id];
        self.builder.build_unconditional_branch(cond_block)?;
        Ok(())
    }
}