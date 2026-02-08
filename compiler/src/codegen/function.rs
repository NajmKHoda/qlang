use std::iter::once;

use inkwell::types::{BasicMetadataTypeEnum, BasicType, StructType};
use inkwell::values::{AnyValue, BasicMetadataValueEnum, BasicValue, FunctionValue, ValueKind};

use super::{CodeGen, CodeGenError};
use crate::codegen::data::GenValue;
use crate::semantics::{BuiltinFunction, BuiltinMethod, Ownership, SemanticClosure, SemanticExpression, SemanticFunction, SemanticTypeKind};

pub(super) struct GenClosureInfo<'ctxt> {
	pub(super) llvm_fn: FunctionValue<'ctxt>,
	pub(super) llvm_context_type: StructType<'ctxt>,
}

impl<'ctxt> CodeGen<'ctxt> {
	pub(super) fn declare_function(&mut self, function: &SemanticFunction) -> Result<(), CodeGenError> {
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
		Ok(())
	}

	pub(super) fn define_function(&mut self, function: &SemanticFunction) -> Result<(), CodeGenError> {
		let llvm_fn = self.llvm_functions[&function.id];
		let entry_block = self.context.append_basic_block(llvm_fn, "entry");
		self.builder.position_at_end(entry_block);

		for (i, param) in function.params.iter().enumerate() {
			let param_var = &self.program.variables[&param.variable_id];
			let llvm_param_val = llvm_fn.get_nth_param(i as u32).unwrap();
			let llvm_param_var = self.builder.build_alloca(
				self.llvm_basic_type(&param.sem_type),
				&param_var.name
			)?;
			self.builder.build_store(llvm_param_var, llvm_param_val)?;
			self.llvm_variables.insert(param.variable_id, llvm_param_var);
		}

		for stmt in &function.body.statements {
			self.gen_stmt(stmt)?;
		}

		Ok(())
	}

    pub fn gen_direct_call(&mut self, function_id: u32, args: &[SemanticExpression]) -> Result<GenValue<'ctxt>, CodeGenError> {
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

	pub fn declare_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let captured_llvm_types = closure.captured_variables.iter()
			.map(|(var_id, _)| {
				let var = &self.program.variables[var_id];
				self.llvm_basic_type(&var.sem_type)
			})
			.collect::<Vec<_>>();
		
		// Generate necessary LLVM types
		let llvm_context_type = self.context.opaque_struct_type(&format!("__ql__context_{}", closure.id));
		llvm_context_type.set_body(&captured_llvm_types, false);
		let mut llvm_param_types: Vec<BasicMetadataTypeEnum> = vec![self.ptr_type().into()];
		llvm_param_types.extend(
			closure.parameters.iter()
			.map(|p| BasicMetadataTypeEnum::from(self.llvm_basic_type(&p.sem_type)))
		);
		let llvm_fn_type = match closure.return_type.kind() {
			SemanticTypeKind::Void => {
				self.void_type().fn_type(&llvm_param_types, false)
			}
			_ => {
				let llvm_return_type = self.llvm_basic_type(&closure.return_type);
				llvm_return_type.fn_type(&llvm_param_types, false)
			},
		};

		let fn_name = format!("__ql__closure_{}", closure.id);
		let llvm_fn = self.module.add_function(&fn_name, llvm_fn_type, None);

		self.closure_info.insert(closure.id, GenClosureInfo {
			llvm_fn,
			llvm_context_type,
		});

		Ok(())
	}

	pub fn define_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let closure_info = &self.closure_info[&closure.id];
		let entry_block = self.context.append_basic_block(closure_info.llvm_fn, "entry");
		self.builder.position_at_end(entry_block);
		
		// Set up captured variable pointers
		let context_ptr = closure_info.llvm_fn.get_nth_param(0).unwrap().into_pointer_value();
		for (i, (var_id, _)) in closure.captured_variables.iter().enumerate() {
			let var = &self.program.variables[var_id];
			let field_ptr = self.builder.build_struct_gep(
				closure_info.llvm_context_type,
				context_ptr,
				i as u32,
				&format!("__captured__{}", var.name)
			)?;
			self.llvm_variables.insert(*var_id, field_ptr);
		}

		// Set up parameter pointers
		for (i, param) in closure.parameters.iter().enumerate() {
			let param_var = &self.program.variables[&param.variable_id];
			let llvm_param_val = closure_info.llvm_fn.get_nth_param((i + 1) as u32).unwrap();
			let llvm_param_var = self.builder.build_alloca(
				self.llvm_basic_type(&param.sem_type),
				&param_var.name
			)?;
			self.builder.build_store(llvm_param_var, llvm_param_val)?;
			self.llvm_variables.insert(param.variable_id, llvm_param_var);
		}

		// Generate closure body
		for stmt in &closure.body.statements {
			self.gen_stmt(stmt)?;
		}

		Ok(())
	}

	pub fn gen_closure_instance(&mut self, closure_id: u32) -> Result<GenValue<'ctxt>, CodeGenError> {
		let closure = &self.program.closures[&closure_id];
		let closure_info = &self.closure_info[&closure_id];

		// Allocate context struct and populate captured variables
		let context_ptr = self.builder.build_malloc(closure_info.llvm_context_type, "closure_context")?;
		for (i, (_, captured_id)) in closure.captured_variables.iter().enumerate() {
			let var_ptr = self.llvm_variables[&captured_id];
			let var_type = &self.program.variables[&captured_id].sem_type;
			let var_llvm_type = self.llvm_basic_type(var_type);
			let field_ptr = self.builder.build_struct_gep(
				closure_info.llvm_context_type,
				context_ptr,
				i as u32,
				&format!("__captured__{}", self.program.variables[captured_id].name)
			)?;

			let var_value = self.builder.build_load(var_llvm_type, var_ptr, "load_captured")?;
			self.builder.build_store(field_ptr, var_value)?;
		}

		// Create callable struct instance
		let callable_ptr = self.builder.build_alloca(self.callable_struct_type, "callable")?;
		let fn_ptr = closure_info.llvm_fn.as_global_value().as_pointer_value();
		let callable_fn_ptr = self.builder.build_struct_gep(
			self.callable_struct_type,
			callable_ptr,
			0,
			"fn_ptr_gep"
		)?;
		self.builder.build_store(callable_fn_ptr, fn_ptr)?;
		let callable_context_ptr = self.builder.build_struct_gep(
			self.callable_struct_type,
			callable_ptr,
			1,
			"context_ptr_gep"
		)?;
		self.builder.build_store(callable_context_ptr, context_ptr)?;

		let callable_struct = self.builder.build_load(
			self.callable_struct_type,
			callable_ptr,
			"load_callable"
		)?.into_struct_value();
		Ok(GenValue::Callable(callable_struct))
	}

	pub fn gen_indirect_call(&mut self, function_expr: &SemanticExpression, args: &[SemanticExpression]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let SemanticTypeKind::Callable(param_types, return_type) = function_expr.sem_type.kind() else {
			panic!("Expected callable type for indirect call");
		};

		let callable_value = self.gen_eval(function_expr)?;
		
		let callable_struct = callable_value.as_llvm_basic_value().into_struct_value();

		let fn_ptr = self.builder
			.build_extract_value(callable_struct, 0, "callable_fn_ptr_gep")?
			.into_pointer_value();
		let context_ptr = self.builder
			.build_extract_value(callable_struct, 1, "callable_context_ptr_gep")?
			.into_pointer_value();
		let arg_values = once(Ok(context_ptr.into()))
			.chain(args.iter()
			.map(|arg| self.gen_eval(arg).map(|v| v.as_llvm_basic_value().into())))
			.collect::<Result<Vec<BasicMetadataValueEnum>, CodeGenError>>()?;

		let llvm_param_types = once(self.ptr_type().into())
			.chain(param_types.iter().map(|t| self.llvm_basic_type(t).into()))
			.collect::<Vec<BasicMetadataTypeEnum>>();
		let llvm_fn_type = match return_type.kind() {
			SemanticTypeKind::Void => {
				self.void_type().fn_type(&llvm_param_types, false)
			}
			_ => {
				let llvm_return_type = self.llvm_basic_type(&return_type);
				llvm_return_type.fn_type(&llvm_param_types, false)
			},
		};

		let call_site = self.builder
			.build_indirect_call(llvm_fn_type, fn_ptr, &arg_values, "indirect_call")?
			.try_as_basic_value();

		match call_site {
			ValueKind::Basic(value) => Ok(GenValue::new(
				&return_type,
				value,
				Ownership::Owned
			)),
			ValueKind::Instruction(_) => Ok(GenValue::Void),
		}
	}
}
