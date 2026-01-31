use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{AnyValue, BasicMetadataValueEnum, BasicValue, ValueKind};

use super::{CodeGen, CodeGenError};
use crate::codegen::data::GenValue;
use crate::semantics::{BuiltinFunction, BuiltinMethod, Ownership, SemanticExpression, SemanticFunction, SemanticTypeKind};

impl<'ctxt> CodeGen<'ctxt> {
	pub(super) fn define_function(&mut self, function: &SemanticFunction) -> Result<(), CodeGenError> {
		let llvm_param_types = function.params.iter()
			.map(|p| self.llvm_basic_type(&p.sem_type).into())
			.collect::<Vec<BasicMetadataTypeEnum>>(); 
		let llvm_type = match function.return_type.kind() {
			SemanticTypeKind::Void => {
				self.void_type().fn_type(&llvm_param_types, false)
			}
			_ => {
				let llvm_return_type = self.llvm_basic_type(&function.return_type);
				llvm_return_type.fn_type(&llvm_param_types, false)
			},
		};

		let llvm_name = if function.name == "main" { "__ql__user_main" } else { &function.name };
		let llvm_fn = self.module.add_function(llvm_name, llvm_type, None);
		self.llvm_functions.insert(function.id, llvm_fn);
		for (i, param) in function.params.iter().enumerate() {
			let llvm_param = llvm_fn.get_nth_param(i as u32).unwrap();
			self.llvm_variables.insert(param.variable_id, llvm_param.into_pointer_value());
		}

		let entry_block = self.context.append_basic_block(llvm_fn, "entry");
		self.builder.position_at_end(entry_block);
		for stmt in &function.body.statements {
			self.gen_stmt(stmt)?;
		}

		Ok(())
	}

    pub fn gen_call(&mut self, function_id: u32, args: &[SemanticExpression]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let sem_function = &self.program.functions[&function_id];
		let llvm_function = self.llvm_functions[&function_id];
		let arg_values = args
			.iter()
			.map(|arg| self.gen_eval(arg))
			.collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;
		let llvm_arg_values = arg_values
			.iter()
			.map(|val| val.as_llvm_basic_value().into())
			.collect::<Vec<BasicMetadataValueEnum>>();

		let call_site = self.builder.build_call(llvm_function, &llvm_arg_values, "call")?;
		for arg in arg_values {
			self.remove_if_owned(arg)?;
		}

		match call_site.try_as_basic_value() {
			ValueKind::Basic(value) => Ok(GenValue::new(
				&sem_function.return_type,
				value,
				Ownership::Owned
			)),
			ValueKind::Instruction(_) => Ok(GenValue::Void),
		}
    }

	pub fn gen_builtin_call(&mut self, function: BuiltinFunction, args: &[SemanticExpression]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let arg_values = args
			.iter()
			.map(|arg| self.gen_eval(arg))
			.collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;
		
		match function {
			BuiltinFunction::PrintString => {
				let str_val = &arg_values[0];
				self.builder.build_call(
					self.runtime_functions.print_string.into(),
					&[str_val.as_llvm_basic_value().into()],
					"print_string"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::PrintInteger => {
				let int_val = &arg_values[0];
				self.builder.build_call(
					self.runtime_functions.print_integer.into(),
					&[int_val.as_llvm_basic_value().into()],
					"print_integer"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::PrintBool => {
				let bool_val = &arg_values[0];
				self.builder.build_call(
					self.runtime_functions.print_boolean.into(),
					&[bool_val.as_llvm_basic_value().into()],
					"print_boolean"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::InputString => {
				let input = self.builder.build_call(
					self.runtime_functions.input_string.into(),
					&[],
					"input_string"
				)?.as_any_value_enum().into_pointer_value();
				Ok(GenValue::String {
					value: input,
					ownership: Ownership::Owned
				})
			}
			BuiltinFunction::InputInteger => {
				let input = self.builder.build_call(
					self.runtime_functions.input_integer.into(),
					&[],
					"input_integer"
				)?.as_any_value_enum().into_int_value();
				Ok(GenValue::Integer(input))
			}
		}
	}

	pub fn gen_return(&mut self, value: &Option<SemanticExpression>) -> Result<(), CodeGenError> {
		let return_value: Option<&dyn BasicValue> = match value {
			Some(val) => {
				let return_val = self.gen_eval(val)?;
				self.add_ref(&return_val)?;
				Some(&return_val.as_llvm_basic_value())
			}
			None => {
				None
			}
		};
		self.builder.build_return(return_value)?;
		Ok(())
	}

	pub fn gen_method_call(
		&mut self,
		object: GenValue<'ctxt>,
		method: BuiltinMethod,
		args: &[SemanticExpression]
	) -> Result<GenValue<'ctxt>, CodeGenError> {
		let mut arg_vals = args.iter()
			.map(|arg| self.gen_eval(arg))
			.collect::<Result<Vec<GenValue<'ctxt>>, CodeGenError>>()?;
		match method {
			BuiltinMethod::ArrayAppend => {
				let elem = arg_vals.remove(0);
				self.gen_array_append(object, elem)
			}
			BuiltinMethod::ArrayLength => {
				self.gen_array_length(object)
			}
			BuiltinMethod::ArrayPop => {
				self.gen_array_pop(object)
			}
		}
	}
}
