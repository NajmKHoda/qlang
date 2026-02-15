use inkwell::{types::{BasicMetadataTypeEnum, BasicType, StructType}, values::{AnyValue, BasicMetadataValueEnum, BasicValue, FunctionValue, GlobalValue, ValueKind}};

use crate::{codegen::{CodeGen, CodeGenError, data::GenValue}, semantics::{Ownership, SemanticClosure, SemanticClosureBody, SemanticExpression, SemanticQuery, SemanticTypeKind}};

pub(super) struct GenClosureInfo<'ctxt> {
	pub(super) llvm_fn: FunctionValue<'ctxt>,
	pub(super) context_type: Option<StructType<'ctxt>>,
	pub(super) context_type_info: Option<GlobalValue<'ctxt>>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn declare_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		// Closure context (captured variables)
		let captured_llvm_types = closure.captured_variables.iter()
			.map(|(var_id, _)| &self.program.variables[var_id].sem_type)
			.collect::<Vec<_>>();
		let (context_type, context_type_info) = if !captured_llvm_types.is_empty() {
			let (_type, _type_info) = self.gen_struct_type_info(
				&format!("__ql__context_{}", closure.id),
				captured_llvm_types.as_slice()
			)?;
			(Some(_type), Some(_type_info))
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
			context_type,
			context_type_info,
		});

		Ok(())
	}

	pub fn define_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let closure_info = &self.closure_info[&closure.id];
		self.cur_fn = Some(closure_info.llvm_fn);
		let entry_block = self.context.append_basic_block(closure_info.llvm_fn, "entry");
		self.builder.position_at_end(entry_block);
		
		// Set up captured variable pointers
		if let Some(context_type) = closure_info.context_type {
			let context_ptr = closure_info.llvm_fn.get_nth_param(0).unwrap().into_pointer_value();
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
			let llvm_param_val = closure_info.llvm_fn.get_nth_param((i + 2) as u32).unwrap();
			let llvm_param_var = self.builder.build_alloca(
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
                let prepared_stmt = closure_info.llvm_fn
					.get_nth_param(1)
					.unwrap()
					.into_pointer_value();
                let result = self.execute_query(prepared_stmt, query)?;
                let return_value = match result {
                    GenValue::Void => None,
                    _ => Some(&result.as_llvm_basic_value() as &dyn BasicValue),
                };
                self.builder.build_return(return_value)?;
            },
        }
		self.cur_fn = None;
		Ok(())
	}

	pub fn gen_callable(&mut self, closure_id: u32) -> Result<GenValue<'ctxt>, CodeGenError> {
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
				callable_type.into(),
				context_type_info.into(),
			],
			"callable_new"
		)?.as_any_value_enum().into_pointer_value();
		
		// Allocate context struct and populate captured variables
		for (i, (_, captured_id)) in closure.captured_variables.iter().enumerate() {
			let variable = &self.program.variables[captured_id];
			let variable_ptr = self.llvm_variables[&captured_id];

			let variable_val = self.load_var(*captured_id)?;
			self.add_ref(&variable_val)?;
			self.builder.build_call(
				self.runtime.callable_capture,
				&[
					callable_ptr.into(),
					self.int_type().const_int(i as u64, false).into(),
					variable_ptr.into(),
				],
				&format!("callable_capture_{}", variable.name)
			)?;
		}

        if let SemanticClosureBody::Query(ref query) = closure.body {
            // For query closures, prepare the statement and store it in the context
            let prepared_stmt = self.prepare_query(query)?;
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

	pub fn gen_indirect_call(&mut self, function_expr: &SemanticExpression, args: &[SemanticExpression]) -> Result<GenValue<'ctxt>, CodeGenError> {
		let SemanticTypeKind::Callable(param_types, return_type) = &function_expr.sem_type.kind() else {
			panic!("Expected callable type for indirect call");
		};

		let callable_ptr = self.gen_eval(function_expr)?
			.as_llvm_basic_value().into_pointer_value();

		let fn_ptr = self.builder.build_call(
			self.runtime.callable_get_fn,
			&[callable_ptr.into()],
			"callable_get_fn"
		)?.as_any_value_enum().into_pointer_value();

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
        
		let llvm_fn_type = match return_type.kind() {
			SemanticTypeKind::Void => {
				self.void_type().fn_type(&llvm_param_types, false)
			}
			_ => {
				let llvm_return_type = self.llvm_basic_type(return_type);
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