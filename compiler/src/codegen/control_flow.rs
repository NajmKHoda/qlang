use inkwell::{types::BasicMetadataTypeEnum, values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, ValueKind}};

use super::{CodeGen, CodeGenError, QLValue};
use crate::{codegen::QLType, tokens::{ExpressionNode, StatementNode}};

pub(super) struct QLFunction<'ctxt> {
    llvm_function: FunctionValue<'ctxt>,
    param_types: Vec<QLType>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn declare_function(
        &mut self,
        name: &str,
        ret_type: QLType,
        param_types: Vec<QLType>,
    ) -> Result<FunctionValue<'ctxt>, CodeGenError> {
        let llvm_param_types = param_types
            .iter()
            .map(|t| self.try_get_nonvoid_type(t)
            .map(BasicMetadataTypeEnum::from))
            .collect::<Result<Vec<BasicMetadataTypeEnum>, CodeGenError>>()?;
        let fn_type = match ret_type {
            QLType::Integer => self.int_type().fn_type(&llvm_param_types, false),
            QLType::Bool => self.bool_type().fn_type(&llvm_param_types, false),
            QLType::Void => self.void_type().fn_type(&llvm_param_types, false),
        };

        let llvm_function = self.module.add_function(name, fn_type, None);
        let function_desc = QLFunction {
            llvm_function,
            param_types
        };

        self.functions.insert(name.to_string(), function_desc);
        Ok(llvm_function)
    }

    pub fn gen_call(&self, fn_name: &str, args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(function) = self.functions.get(fn_name) {
            if function.param_types.len() != args.len() {
                return Err(CodeGenError::BadFunctionCallError(fn_name.to_string()));
            }
            for (expected_type, arg) in function.param_types.iter().zip(args.iter()) {
                if *expected_type != arg.get_type() {
                    return Err(CodeGenError::BadFunctionCallError(fn_name.to_string()));
                }
            }

            let arg_values: Vec<BasicMetadataValueEnum> = args
                .into_iter()
                .map(|v| BasicValueEnum::try_from(v).map(BasicMetadataValueEnum::from))
                .collect::<Result<Vec<BasicMetadataValueEnum>, CodeGenError>>()?;
            let call_site = self.builder.build_call(function.llvm_function, &arg_values, "call")?;
            match call_site.try_as_basic_value() {
                ValueKind::Basic(value) => Ok(value.try_into()?),
                ValueKind::Instruction(_) => Ok(QLValue::Void),
            }
        } else {
            Err(CodeGenError::UndefinedVariableError(fn_name.to_string()))
        }
    }

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
