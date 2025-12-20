use inkwell::types::{BasicMetadataTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, ValueKind};

use super::{CodeGen, CodeGenError, QLValue, QLType};
use crate::tokens::TypedQNameNode;

pub(super) struct QLParameter {
	pub(super) name: String,
	pub(super) ql_type: QLType,
}

impl From<TypedQNameNode> for QLParameter {
	fn from(node: TypedQNameNode) -> Self {
		QLParameter {
			name: node.name,
			ql_type: node.ql_type,
		}
	}
}

pub(super) struct QLFunction<'ctxt> {
	pub(super) llvm_function: FunctionValue<'ctxt>,
	pub(super) return_type: QLType,
	pub(super) name: String,
	pub(super) params: Vec<QLParameter>
}

impl<'ctxt> QLFunction<'ctxt> {
	pub fn check_args(&self, args: &[QLValue<'ctxt>]) -> Result<(), CodeGenError> {
		if self.params.len() != args.len() {
			return Err(CodeGenError::BadFunctionCallError(self.name.clone()));
		}
		for (param, arg) in self.params.iter().zip(args.iter()) {
			if param.ql_type != arg.get_type() {
				return Err(CodeGenError::BadFunctionCallError(self.name.clone()));
			}
		}
		Ok(())
	}

	pub fn try_get_arg_value(&self, name: &str) -> Option<QLValue<'ctxt>> {
		let (index, param) = self.params.iter().enumerate().find(|(_, p)| p.name == name)?;
		let llvm_value = self.llvm_function.get_nth_param(index as u32)?;
		Some(param.ql_type.to_value(llvm_value, true))
	}
}

impl<'ctxt> CodeGen<'ctxt> {
	pub (super) fn get_fn_type(&self, return_type: QLType, param_types: &[QLType]) -> Result<FunctionType<'ctxt>, CodeGenError> {
		let llvm_param_types = param_types
			.iter()
			.map(|t| self.try_get_nonvoid_type(t)
			.map(BasicMetadataTypeEnum::from))
			.collect::<Result<Vec<BasicMetadataTypeEnum>, CodeGenError>>()?;

		let fn_type = match return_type {
			QLType::Integer => self.int_type().fn_type(&llvm_param_types, false),
			QLType::Bool => self.bool_type().fn_type(&llvm_param_types, false),
			QLType::String => self.ptr_type().fn_type(&llvm_param_types, false),
			QLType::Void => self.void_type().fn_type(&llvm_param_types, false),
		};
		Ok(fn_type)
	}

	pub(super) fn declare_user_function(
		&mut self,
		name: &str,
		return_type: QLType,
		param_nodes: Vec<TypedQNameNode>,
	) -> Result<&QLFunction<'ctxt>, CodeGenError> {
		let params: Vec<QLParameter> = param_nodes.into_iter().map(|n| n.into()).collect();
		let param_types: Vec<QLType> = params.iter().map(|p| p.ql_type).collect();
		let fn_type = self.get_fn_type(return_type, &param_types)?;

		self.functions.insert(name.to_string(), QLFunction {
			name: name.to_string(),
			llvm_function: self.module.add_function(name, fn_type, None),
			return_type,
			params
		});

		Ok(self.functions.get(name).unwrap())
	}

    pub fn gen_call(&self, fn_name: &str, args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(function) = self.functions.get(fn_name) {
            function.check_args(&args)?;
            let arg_values: Vec<BasicMetadataValueEnum> = args
                .iter()
                .map(|v| BasicValueEnum::try_from(*v).map(BasicMetadataValueEnum::from))
                .collect::<Result<Vec<BasicMetadataValueEnum>, CodeGenError>>()?;

            let call_site = self.builder.build_call(function.llvm_function, &arg_values, "call")?;
			for arg in args {
				self.remove_if_temp(arg)?;
			}

            match call_site.try_as_basic_value() {
                ValueKind::Basic(value) => Ok(function.return_type.to_value(value, false)),
                ValueKind::Instruction(_) => Ok(QLValue::Void),
            }
        } else {
            Err(CodeGenError::UndefinedVariableError(fn_name.to_string()))
        }
    }

	pub fn gen_return(&mut self, value: Option<QLValue<'ctxt>>) -> Result<(), CodeGenError> {
		let return_type = self.cur_fn().return_type;
		match value {
			Some(val) => {
				if val.get_type() != return_type {
					return Err(CodeGenError::UnexpectedTypeError);
				}
				let basic_value = BasicValueEnum::try_from(val)?;
				self.builder.build_return(Some(&basic_value))
			}
			None => {
				if return_type != QLType::Void {
					return Err(CodeGenError::UnexpectedTypeError);
				}
				self.builder.build_return(None)
			}
		}?;
		Ok(())
	}
}
