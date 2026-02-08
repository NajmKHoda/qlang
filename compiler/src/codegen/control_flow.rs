use inkwell::{basic_block::BasicBlock, values::{BasicValue, IntValue}};

use super::{CodeGen, CodeGenError};
use crate::semantics::{SemanticBlock, SemanticConditionalBranch, SemanticExpression, SemanticTypeKind};

pub(super) struct GenLoopInfo<'a> {
    cond_block: BasicBlock<'a>,
    after_block: BasicBlock<'a>
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

        if !block.terminates {
            for var_id in &self.vars_to_drop {
                self.drop_var(*var_id)?;
            }
            self.vars_to_drop.clear();
        } 
        
        Ok(())
    }

    pub fn gen_return(&mut self, value: &Option<SemanticExpression>) -> Result<(), CodeGenError> {
		let return_value: Option<&dyn BasicValue> = match value {
			Some(val) => {
				let return_val = self.gen_eval(val)?;
				if val.sem_type.kind() != SemanticTypeKind::Void {
					self.add_ref(&return_val)?;
					Some(&return_val.as_llvm_basic_value())
				} else {
					None
				}
			}
			None => None
		};

		for var_id in &self.vars_to_drop {
			self.drop_var(*var_id)?;
		}
        self.vars_to_drop.clear();
		self.builder.build_return(return_value)?;
		Ok(())
	}

    pub fn gen_break(&mut self, loop_id: u32) -> Result<(), CodeGenError> {
        let GenLoopInfo { after_block, .. } = self.loop_info[&loop_id];
        for var_id in &self.vars_to_drop {
            self.drop_var(*var_id)?;
        }
        self.vars_to_drop.clear();
        self.builder.build_unconditional_branch(after_block)?;
        Ok(())
    }

    pub fn gen_continue(&mut self, loop_id: u32) -> Result<(), CodeGenError> {
        let GenLoopInfo { cond_block, .. } = self.loop_info[&loop_id];
        for var_id in &self.vars_to_drop {
            self.drop_var(*var_id)?;
        }
        self.vars_to_drop.clear();
        self.builder.build_unconditional_branch(cond_block)?;
        Ok(())
    }
}