use inkwell::types::{BasicMetadataTypeEnum, BasicType, FunctionType};
use inkwell::values::{AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, ValueKind};

use super::{CodeGen, CodeGenError, QLValue, QLType};
use crate::tokens::TypedQNameNode;

pub(super) struct QLParameter {
	pub(super) name: String,
	pub(super) ql_type: QLType,
}

impl From<&TypedQNameNode> for QLParameter {
	fn from(node: &TypedQNameNode) -> Self {
		QLParameter {
			name: node.name.clone(),
			ql_type: node.ql_type.clone(),
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
	pub (super) fn get_fn_type(&self, return_type: &QLType, param_types: &[QLType]) -> Result<FunctionType<'ctxt>, CodeGenError> {
		let llvm_param_types = param_types
			.iter()
			.map(|t| self.try_get_nonvoid_type(t)
			.map(BasicMetadataTypeEnum::from))
			.collect::<Result<Vec<BasicMetadataTypeEnum>, CodeGenError>>()?;

		let fn_type = match return_type {
			QLType::Void => self.void_type().fn_type(&llvm_param_types, false),
			_ => self.try_get_nonvoid_type(return_type)?.fn_type(&llvm_param_types, false)
		};
		Ok(fn_type)
	}

	pub(super) fn declare_user_function(
		&mut self,
		name: &str,
		return_type: &QLType,
		param_nodes: &[TypedQNameNode],
	) -> Result<&QLFunction<'ctxt>, CodeGenError> { 
		let params: Vec<QLParameter> = param_nodes.iter().map(|n| n.into()).collect();
		let param_types: Vec<QLType> = params.iter().map(|p| p.ql_type.clone()).collect();
		let fn_type = self.get_fn_type(return_type, &param_types)?;

		let llvm_name = if name == "main" { "__ql__user_main" } else { name };
		self.functions.insert(name.to_string(), QLFunction {
			name: name.to_string(),
			llvm_function: self.module.add_function(llvm_name, fn_type, None),
			return_type: return_type.clone(),
			params
		});

		Ok(self.functions.get(name).unwrap())
	}

    pub fn gen_call(&self, fn_name: &str, args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(function) = self.functions.get(fn_name) {
            function.check_args(&args)?;
            let arg_values: Vec<BasicMetadataValueEnum> = args
                .iter()
                .map(|v| BasicValueEnum::try_from(v.clone()).map(BasicMetadataValueEnum::from))
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
		let return_type = self.cur_fn().return_type.clone();
		let enum_value = match value {
			Some(val) => {
				if val.get_type() != return_type {
					return Err(CodeGenError::UnexpectedTypeError);
				}
				self.add_ref(&val)?;
				let basic_value = BasicValueEnum::try_from(val)?;
				Some(basic_value)
			}
			None => {
				if return_type != QLType::Void {
					return Err(CodeGenError::UnexpectedTypeError);
				}
				None
			}
		};

		
		for scope in self.scopes.iter().rev() {
			self.release_scope(scope)?;
		}

		let basic_value = enum_value.as_ref().map(|v| v as &dyn BasicValue);
		self.builder.build_return(basic_value)?;
		Ok(())
	}

	pub fn gen_method_call(&self, object: QLValue<'ctxt>, method_name: &str, mut args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
		match object {
			QLValue::Array(array_ptr, elem_type, _) => {
				match method_name {
					"append" => {
						if args.len() != 1 {
							return Err(CodeGenError::BadFunctionCallError("Array.append".to_string()));
						}
						let elem = args.remove(0);
						if elem.get_type() != elem_type {
							return Err(CodeGenError::UnexpectedTypeError);
						}
						let elem_basic: BasicValueEnum = elem.clone().try_into()?;
						let elem_ptr = self.builder.build_alloca(
							self.try_get_nonvoid_type(&elem_type)?,
							"append_elem_ptr"
						)?;
						self.builder.build_store(elem_ptr, elem_basic)?;

						self.builder.build_call(
							self.runtime_functions.append_array.into(),
							&[array_ptr.into(), elem_ptr.into()],
							"array_append"
						)?;

						self.remove_if_temp(elem)?;
						Ok(QLValue::Void)
					}
					"length" => {
						if !args.is_empty() {
							return Err(CodeGenError::BadFunctionCallError("Array.length".to_string()));
						}
						let length_value = self.builder.build_call(
							self.runtime_functions.array_length.into(),
							&[array_ptr.into()],
							"array_length"
						)?.as_any_value_enum().into_int_value();

						Ok(QLValue::Integer(length_value))
					}
					"pop" => {
						if !args.is_empty() {
							return Err(CodeGenError::BadFunctionCallError("Array.pop".to_string()));
						}

						let elem_ptr = self.builder.build_call(
							self.runtime_functions.pop_array.into(),
							&[array_ptr.into()],
							"array_pop"
						)?.as_any_value_enum().into_pointer_value();

						let loaded_elem = self.builder.build_load(
							self.try_get_nonvoid_type(&elem_type)?,
							elem_ptr,
							"pop_elem_load"
						)?;

						Ok(elem_type.to_value(loaded_elem, false))
					}
					_ => Err(CodeGenError::UndefinedMethodError(method_name.to_string(), "Array".to_string())),
				}
			}
			_ => Err(CodeGenError::UndefinedMethodError(
				method_name.to_string(),
				format!("{:?}", object.get_type())
			)),
		}
	}
}
