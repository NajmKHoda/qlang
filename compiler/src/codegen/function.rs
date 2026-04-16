use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{AnyValue, BasicMetadataValueEnum, ValueKind};

use super::{CodeGen, CodeGenError};
use crate::codegen::data::GenValue;
use crate::semantics::{BuiltinFunction, BuiltinMethod, Ownership, SemanticExpression, SemanticFunction, SemanticTypeKind};

impl<'ctxt> CodeGen<'ctxt> {
	pub(super) fn declare_function(&mut self, function: &SemanticFunction) -> Result<(), CodeGenError> {
		let llvm_param_types = function.param_ids.iter()
			.map(|p| self.llvm_basic_type(&self.program.variables[p].sem_type).into())
			.collect::<Vec<BasicMetadataTypeEnum>>(); 
		let llvm_type = if function.is_failable {
			let result_ty = self.llvm_result_type(&function.return_type);
			result_ty.fn_type(&llvm_param_types, false)
		} else {
			match function.return_type.kind() {
				SemanticTypeKind::Void => self.void_type().fn_type(&llvm_param_types, false),
				_ => {
					let llvm_return_type = self.llvm_basic_type(&function.return_type);
					llvm_return_type.fn_type(&llvm_param_types, false)
				}
			}
		};

		let llvm_name = if function.name == "main" { "__ql__user_main" } else { &function.name };
		let llvm_fn = self.module.add_function(llvm_name, llvm_type, None);
		self.llvm_functions.insert(function.id, llvm_fn);
		Ok(())
	}

	pub(super) fn define_function(&mut self, function: &SemanticFunction) -> Result<(), CodeGenError> {
		let llvm_fn = self.llvm_functions[&function.id];

		self.cur_fn = Some(llvm_fn);
		self.cur_function_id = Some(function.id);
		self.cur_closure_id = None;
		let entry_block = self.context.append_basic_block(llvm_fn, "entry");
		self.builder.position_at_end(entry_block);

		for (i, param_id) in function.param_ids.iter().enumerate() {
			let param_var = &self.program.variables[param_id];
			let llvm_param_val = llvm_fn.get_nth_param(i as u32).unwrap();
			let llvm_param_var = self.build_alloca(
				self.llvm_basic_type(&param_var.sem_type),
				&param_var.name
			)?;
			self.builder.build_store(llvm_param_var, llvm_param_val)?;
			self.llvm_variables.insert(*param_id, llvm_param_var);
		}
		self.gen_block(&function.body)?;
		
		self.cur_fn = None;
		self.cur_function_id = None;
		Ok(())
	}

	fn gen_error_drops(&self, error_drops: &[u32]) -> Result<(), CodeGenError> {
		for variable_id in error_drops {
			if let Some(variable_ptr) = self.llvm_variables.get(variable_id) {
				let sem_type = &self.program.variables[variable_id].sem_type;
				self.drop_value(*variable_ptr, sem_type)?;
			}
		}
		Ok(())
	}

	fn gen_failable_error_return(&self, error_drops: &[u32]) -> Result<(), CodeGenError> {
		self.gen_error_drops(error_drops)?;

		let return_type = self.cur_executable_return_type();
		let result_ty = self.llvm_result_type(&return_type);
		let result_alloca = self.build_alloca(result_ty.into(), "failable_error_result")?;

		let is_error_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 0, "result_is_error_ptr")?;
		self.builder.build_store(is_error_ptr, self.bool_type().const_int(1, false))?;

		let result_val = self.builder.build_load(result_ty, result_alloca, "failable_error_result_val")?.into_struct_value();
		self.builder.build_return(Some(&result_val))?;
		Ok(())
	}

	pub(super) fn gen_failable_check(
		&mut self,
		is_error: inkwell::values::IntValue<'ctxt>,
		error_drops: &[u32],
	) -> Result<inkwell::basic_block::BasicBlock<'ctxt>, CodeGenError> {
		let cur_block = self.builder.get_insert_block().unwrap();
		let cur_fn = self.cur_fn.unwrap();
		let ok_block = self.context.append_basic_block(cur_fn, "failable_ok");
		let err_block = self.context.append_basic_block(cur_fn, "failable_err");

		self.builder.position_at_end(cur_block);
		self.builder.build_conditional_branch(is_error, err_block, ok_block)?;

		self.builder.position_at_end(err_block);
		self.gen_error_drops(error_drops)?;
		if let Some(txn) = self.transaction_stack.last() {
			self.builder.build_unconditional_branch(txn.rollback_block)?;
		} else {
			self.gen_failable_error_return(&[])?;
		}

		self.builder.position_at_end(ok_block);
		Ok(ok_block)
	}

	pub fn gen_direct_call(&mut self, function_id: u32, args: &[SemanticExpression], error_drops: &[u32]) -> Result<GenValue<'ctxt>, CodeGenError> {
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
		for (i, arg) in arg_values.iter().enumerate() {
			if arg.ownership() == Ownership::Owned {
				let param_id = sem_function.param_ids[i];
				let param_type = &self.program.variables[&param_id].sem_type;
				let arg_ptr = self.put_on_stack(&arg, &format!("{}_arg_ptr", sem_function.name))?;
				self.drop_value(arg_ptr, param_type)?;
			}
		}

		if sem_function.is_failable {
			let result_struct = call_site.as_any_value_enum().into_struct_value();
			let is_error = self.builder.build_extract_value(result_struct, 0, "call_is_error")?
				.into_int_value();
			self.gen_failable_check(is_error, error_drops)?;

			if sem_function.return_type == SemanticTypeKind::Void {
				return Ok(GenValue::Void);
			}

			let ok_val = self.builder.build_extract_value(result_struct, 1, "call_ok_value")?;
			Ok(GenValue::new(&sem_function.return_type, ok_val, Ownership::Owned))
		} else {
			match call_site.try_as_basic_value() {
				ValueKind::Basic(value) => Ok(GenValue::new(
					&sem_function.return_type,
					value,
					Ownership::Owned
				)),
				ValueKind::Instruction(_) => Ok(GenValue::Void),
			}
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
					self.runtime.print_string,
					&[str_val.as_llvm_basic_value().into()],
					"print_string"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::PrintInteger => {
				let int_val = &arg_values[0];
				self.builder.build_call(
					self.runtime.print_integer,
					&[int_val.as_llvm_basic_value().into()],
					"print_integer"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::PrintBool => {
				let bool_val = &arg_values[0];
				self.builder.build_call(
					self.runtime.print_boolean,
					&[bool_val.as_llvm_basic_value().into()],
					"print_boolean"
				)?;
				Ok(GenValue::Void)
			}
			BuiltinFunction::InputString => {
				let input = self.builder.build_call(
					self.runtime.input_string,
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
					self.runtime.input_integer,
					&[],
					"input_integer"
				)?.as_any_value_enum().into_int_value();
				Ok(GenValue::Integer(input))
			}
			BuiltinFunction::Zip => {
				let GenValue::Iterator { value: iter_a_ptr, elem_type, .. } = &arg_values[0] else {
					panic!("Expected iterator value");
				};
				let GenValue::Iterator { value: iter_b_ptr, .. } = &arg_values[1] else {
					panic!("Expected iterator value");
				};

				let iter_ptr = self.builder.build_call(
					self.runtime.iterator_zip,
					&[(*iter_a_ptr).into(), (*iter_b_ptr).into()],
					"iterator_zip"
				)?.as_any_value_enum().into_pointer_value();

				Ok(GenValue::Iterator {
					value: iter_ptr,
					elem_type: elem_type.clone(),
					ownership: Ownership::Owned,
				})
			}
			BuiltinFunction::Concat => {
				let GenValue::Iterator { value: iter_a_ptr, elem_type, .. } = &arg_values[0] else {
					panic!("Expected iterator value");
				};
				let GenValue::Iterator { value: iter_b_ptr, .. } = &arg_values[1] else {
					panic!("Expected iterator value");
				};

				let iter_ptr = self.builder.build_call(
					self.runtime.iterator_concat,
					&[(*iter_a_ptr).into(), (*iter_b_ptr).into()],
					"iterator_concat"
				)?.as_any_value_enum().into_pointer_value();

				Ok(GenValue::Iterator {
					value: iter_ptr,
					elem_type: elem_type.clone(),
					ownership: Ownership::Owned,
				})
			}
		}
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
			BuiltinMethod::ArrayIter => {
				self.gen_array_iter(object)
			}
			BuiltinMethod::IteratorNext => {
				self.gen_iterator_next(object)
			}
			BuiltinMethod::IteratorHasNext => {
				self.gen_iterator_has_next(object)
			}
			BuiltinMethod::IteratorCollect => {
				self.gen_iterator_collect(object)
			}
		}
	}
}
