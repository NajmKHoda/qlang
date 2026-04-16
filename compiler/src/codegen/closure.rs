use inkwell::{types::{BasicMetadataTypeEnum, BasicType, StructType}, values::{AnyValue, BasicMetadataValueEnum, FunctionValue, GlobalValue, ValueKind}};

use crate::{codegen::{CodeGen, CodeGenError, data::GenValue}, semantics::{Ownership, SemanticClosure, SemanticClosureBody, SemanticExpression, SemanticQuery, SemanticType, SemanticTypeKind}};

pub(super) struct GenClosureInfo<'ctxt> {
	pub(super) llvm_fn: FunctionValue<'ctxt>,
	pub(super) llvm_failable_fn: FunctionValue<'ctxt>,
	pub(super) context_type: Option<StructType<'ctxt>>,
	pub(super) context_type_info: Option<GlobalValue<'ctxt>>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn declare_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		// Closure context (captured variables)
		let captured_llvm_types = closure.captured_variables.iter()
			.map(|(var_id, _)| self.program.variables[var_id].sem_type.clone())
			.collect::<Vec<SemanticType>>();
		let (context_type, context_type_info) = if !captured_llvm_types.is_empty() {
			let struct_info = self.gen_struct_info(
				&format!("context_{}", closure.id),
				captured_llvm_types.as_slice(),
				false
			)?;
			(Some(struct_info.struct_type), Some(struct_info.type_info))
		} else {
			(None, None)
		};

		let mut llvm_param_types: Vec<BasicMetadataTypeEnum> = vec![self.ptr_type().into(); 2];
		llvm_param_types.extend(
			closure.param_ids.iter()
			.map(|p| BasicMetadataTypeEnum::from(
				self.llvm_basic_type(&self.program.variables[p].sem_type)
			))
		);
		let llvm_fn_type = if closure.is_failable {
			self.llvm_result_type(&closure.return_type).fn_type(&llvm_param_types, false)
		} else {
			match closure.return_type.kind() {
				SemanticTypeKind::Void => self.void_type().fn_type(&llvm_param_types, false),
				_ => {
					let llvm_return_type = self.llvm_basic_type(&closure.return_type);
					llvm_return_type.fn_type(&llvm_param_types, false)
				}
			}
		};

		let fn_name = format!("__ql__closure_{}", closure.id);
		let llvm_fn = self.module.add_function(&fn_name, llvm_fn_type, None);
		let llvm_failable_fn = if closure.is_failable {
			llvm_fn
		} else {
			let wrapper_name = format!("__ql__closure_failable_wrap_{}", closure.id);
			let wrapper_ty = self.llvm_result_type(&closure.return_type).fn_type(&llvm_param_types, false);
			self.module.add_function(&wrapper_name, wrapper_ty, None)
		};

		self.closure_info.insert(closure.id, GenClosureInfo {
			llvm_fn,
			llvm_failable_fn,
			context_type,
			context_type_info,
		});

		Ok(())
	}

	pub fn define_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let closure_info = &self.closure_info[&closure.id];
		let llvm_fn = closure_info.llvm_fn;
		let llvm_failable_fn = closure_info.llvm_failable_fn;
		let context_type_opt = closure_info.context_type;

		self.cur_fn = Some(llvm_fn);
		self.cur_function_id = None;
		self.cur_closure_id = Some(closure.id);
		let entry_block = self.context.append_basic_block(llvm_fn, "entry");
		self.builder.position_at_end(entry_block);
		
		// Set up captured variable pointers
		if let Some(context_type) = context_type_opt {
			let context_ptr = llvm_fn.get_nth_param(0).unwrap().into_pointer_value();
			for (i, (var_id, _)) in closure.captured_variables.iter().enumerate() {
				let var = &self.program.variables[var_id];
				let field_ptr = self.builder.build_struct_gep(
					context_type,
					context_ptr,
					i as u32,
					&format!("__captured__{}", var.name)
				)?;
				self.llvm_variables.insert(*var_id, field_ptr);
			}
		}

		// Set up parameter pointers
		for (i, param_id) in closure.param_ids.iter().enumerate() {
			let param_var = &self.program.variables[param_id];
			let llvm_param_val = llvm_fn.get_nth_param((i + 2) as u32).unwrap();
			let llvm_param_var = self.build_alloca(
				self.llvm_basic_type(&param_var.sem_type),
				&param_var.name
			)?;
			self.builder.build_store(llvm_param_var, llvm_param_val)?;
			self.llvm_variables.insert(*param_id, llvm_param_var);
		}
		
        match closure.body {
            SemanticClosureBody::Procedural(ref body) => self.gen_block(body)?,
            SemanticClosureBody::Query(ref query) => {
                // Prepared statement is the last context field
                let prepared_stmt = llvm_fn
					.get_nth_param(1)
					.unwrap()
					.into_pointer_value();
				let prep_failed = self.builder.build_is_null(prepared_stmt, "query_prepare_failed")?;
				let (result, exec_ok) = self.execute_query(prepared_stmt, query)?;
				let exec_failed = self.builder.build_int_compare(
					inkwell::IntPredicate::EQ,
					exec_ok,
					self.bool_type().const_int(0, false),
					"query_exec_failed",
				)?;
				let is_error = self.builder.build_or(prep_failed, exec_failed, "query_is_error")?;

				let result_ty = self.llvm_result_type(&closure.return_type);
				let result_alloca = self.build_alloca(result_ty.into(), "query_closure_result")?;
				let is_error_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 0, "query_result_is_error_ptr")?;
				self.builder.build_store(is_error_ptr, is_error)?;

				if closure.return_type != SemanticTypeKind::Void {
					let ok_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 1, "query_result_value_ptr")?;
					self.builder.build_store(ok_ptr, result.as_llvm_basic_value())?;
				}

				let result_val = self.builder.build_load(result_ty, result_alloca, "query_result_val")?.into_struct_value();
				self.builder.build_return(Some(&result_val))?;
            },
        }

		if !closure.is_failable {
			let wrap_fn = llvm_failable_fn;
			let wrap_entry = self.context.append_basic_block(wrap_fn, "entry");
			self.cur_fn = Some(wrap_fn);
			self.cur_function_id = None;
			self.cur_closure_id = None;
			self.builder.position_at_end(wrap_entry);

			let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
			for i in 0..(closure.param_ids.len() + 2) {
				call_args.push(wrap_fn.get_nth_param(i as u32).unwrap().into());
			}
			let call_site = self.builder.build_call(llvm_fn, &call_args, "closure_true_call")?;

			let result_ty = self.llvm_result_type(&closure.return_type);
			let result_alloca = self.build_alloca(result_ty.into(), "wrapped_result")?;
			let is_error_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 0, "wrapped_result_is_error")?;
			self.builder.build_store(is_error_ptr, self.bool_type().const_int(0, false))?;

			if closure.return_type != SemanticTypeKind::Void {
				let ok_ptr = self.builder.build_struct_gep(result_ty, result_alloca, 1, "wrapped_result_value")?;
				let ok_val = match call_site.try_as_basic_value() {
					ValueKind::Basic(value) => value,
					ValueKind::Instruction(_) => panic!("Expected non-void closure return in wrapper"),
				};
				self.builder.build_store(ok_ptr, ok_val)?;
			}

			let result_val = self.builder.build_load(result_ty, result_alloca, "wrapped_result_val")?.into_struct_value();
			self.builder.build_return(Some(&result_val))?;
		}

		self.cur_fn = None;
		self.cur_closure_id = None;
		Ok(())
	}

	pub fn gen_callable(&mut self, closure_id: u32, error_drops: &[u32]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let closure = &self.program.closures[&closure_id];
		let closure_info = &self.closure_info[&closure_id];

		// Create the callable
		let callable_type_val = match &closure.body {
			SemanticClosureBody::Procedural(_) => 0, // CallableType::PROCEDURAL
			SemanticClosureBody::Query(query) => match query {
				SemanticQuery::Select { .. } => 1, // CallableType::SELECT
				SemanticQuery::Insert { .. } => 2, // CallableType::INSERT
				SemanticQuery::Update { .. } => 3, // CallableType::UPDATE
				SemanticQuery::Delete { .. } => 4, // CallableType::DELETE
			},
		};
		let callable_type = self.int_type().const_int(callable_type_val, false);
		let context_type_info = match closure_info.context_type_info {
			Some(info) => info.as_pointer_value(),
			None => self.ptr_type().const_null(),
		};
		let callable_ptr = self.builder.build_call(
			self.runtime.callable_new,
			&[
				closure_info.llvm_fn.as_global_value().as_pointer_value().into(),
				closure_info.llvm_failable_fn.as_global_value().as_pointer_value().into(),
				callable_type.into(),
				context_type_info.into(),
			],
			"callable_new"
		)?.as_any_value_enum().into_pointer_value();
		
		// Allocate context struct and populate captured variables
		if closure.captured_variables.len() > 0 {
			let context_ptr = self.builder.build_call(
				self.runtime.callable_get_context,
				&[callable_ptr.into()],
				"callable_get_context"
			)?.as_any_value_enum().into_pointer_value();

			for (i, (_, captured_id)) in closure.captured_variables.iter().enumerate() {
				let variable = &self.program.variables[captured_id];
				let variable_ptr = self.llvm_variables[&captured_id];
				self.copy_value(variable_ptr, &variable.sem_type)?;

				let ctxt_field_ptr = self.builder.build_struct_gep(
					closure_info.context_type.unwrap(),
					context_ptr,
					i as u32,
					&format!("context_{}", variable.name)
				)?;

				let llvm_type = self.llvm_basic_type(&variable.sem_type);
				let variable_val = self.builder.build_load(
					llvm_type,
					variable_ptr,
					&format!("load_captured_{}", variable.name)
				)?;
				self.builder.build_store(ctxt_field_ptr, variable_val)?;
			}
		}

        if let SemanticClosureBody::Query(ref query) = closure.body {
            // For query closures, prepare the statement and store it in the context
            let prepared_stmt = self.prepare_query(query)?;

			let stmt_is_null = self.builder.build_is_null(prepared_stmt, "prepared_stmt_is_null")?;
			let prep_ok_block = self.context.append_basic_block(self.cur_fn.unwrap(), "prep_ok");
			let prep_failed_block = self.context.append_basic_block(self.cur_fn.unwrap(), "prep_failed");
			self.builder.build_conditional_branch(stmt_is_null, prep_failed_block, prep_ok_block)?;

			self.builder.position_at_end(prep_failed_block);
			self.gen_failable_error_return(error_drops)?;

			self.builder.position_at_end(prep_ok_block);
            self.builder.build_call(
				self.runtime.callable_set_stmt,
				&[callable_ptr.into(), prepared_stmt.into()],
				"callable_set_prepared_stmt"
			)?;
        }

		Ok(GenValue::Callable {
			value: callable_ptr,
			ownership: Ownership::Owned
		})
	}

	pub fn gen_indirect_call(&mut self, function_expr: &SemanticExpression, args: &[SemanticExpression], error_drops: &[u32]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let SemanticTypeKind::Callable { is_failable, param_types, ret_type: return_type } = &function_expr.sem_type.kind() else {
			panic!("Expected callable type for indirect call");
		};

		let callable_ptr = self.gen_eval(function_expr)?
			.as_llvm_basic_value().into_pointer_value();

		let fn_ptr = if *is_failable {
			self.builder.build_call(
				self.runtime.callable_get_failable_fn,
				&[callable_ptr.into()],
				"callable_get_failable_fn"
			)?.as_any_value_enum().into_pointer_value()
		} else {
			self.builder.build_call(
				self.runtime.callable_get_fn,
				&[callable_ptr.into()],
				"callable_get_fn"
			)?.as_any_value_enum().into_pointer_value()
		};

		let context_ptr = self.builder.build_call(
			self.runtime.callable_get_context,
			&[callable_ptr.into()],
			"callable_get_context"
		)?.as_any_value_enum().into_pointer_value();

		let prepared_stmt = self.builder.build_call(
			self.runtime.callable_get_stmt,
			&[callable_ptr.into()],
			"callable_get_stmt"
		)?.as_any_value_enum().into_pointer_value();

		let mut llvm_param_types: Vec<BasicMetadataTypeEnum> = vec![self.ptr_type().into(); 2];
		let mut arg_values: Vec<BasicMetadataValueEnum> = vec![context_ptr.into(), prepared_stmt.into()];
		for (arg, param_type) in args.iter().zip(param_types) {
			let arg_val = self.gen_eval(arg)?.as_llvm_basic_value();
			arg_values.push(arg_val.into());
			llvm_param_types.push(self.llvm_basic_type(param_type).into());
		}
        
		let llvm_fn_type = if *is_failable {
			self.llvm_result_type(return_type).fn_type(&llvm_param_types, false)
		} else {
			match return_type.kind() {
				SemanticTypeKind::Void => self.void_type().fn_type(&llvm_param_types, false),
				_ => {
					let llvm_return_type = self.llvm_basic_type(return_type);
					llvm_return_type.fn_type(&llvm_param_types, false)
				}
			}
		};

		let call_site = self.builder.build_indirect_call(llvm_fn_type, fn_ptr, &arg_values, "indirect_call")?;
		if *is_failable {
			let result_struct = call_site.as_any_value_enum().into_struct_value();
			let is_error = self.builder.build_extract_value(result_struct, 0, "callable_call_is_error")?
				.into_int_value();
			self.gen_failable_check(is_error, error_drops)?;

			if return_type.kind() == SemanticTypeKind::Void {
				Ok(GenValue::Void)
			} else {
				let ok_val = self.builder.build_extract_value(result_struct, 1, "callable_call_ok")?;
				Ok(GenValue::new(return_type, ok_val, Ownership::Owned))
			}
		} else {
			match call_site.try_as_basic_value() {
				ValueKind::Basic(value) => Ok(GenValue::new(return_type, value, Ownership::Owned)),
				ValueKind::Instruction(_) => Ok(GenValue::Void),
			}
		}
	}
}