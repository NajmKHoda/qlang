use std::iter::once;

use inkwell::{types::{BasicMetadataTypeEnum, BasicType, StructType}, values::{BasicMetadataValueEnum, BasicValue, FunctionValue, ValueKind}};

use crate::{codegen::{CodeGen, CodeGenError, data::GenValue}, semantics::{Ownership, SemanticClosure, SemanticClosureBody, SemanticExpression, SemanticTypeKind}};

pub(super) struct GenClosureInfo<'ctxt> {
	pub(super) llvm_fn: FunctionValue<'ctxt>,
	pub(super) llvm_context_type: StructType<'ctxt>,
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn declare_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let mut captured_llvm_types = closure.captured_variables.iter()
			.map(|(var_id, _)| {
				let var = &self.program.variables[var_id];
				self.llvm_basic_type(&var.sem_type)
			})
			.collect::<Vec<_>>();
		if closure.body.is_query() {
			captured_llvm_types.push(self.ptr_type().into());
		}
		
		// Generate necessary LLVM types
		let llvm_context_type = self.context.opaque_struct_type(&format!("__ql__context_{}", closure.id));
		llvm_context_type.set_body(&captured_llvm_types, false);
		let mut llvm_param_types: Vec<BasicMetadataTypeEnum> = vec![self.ptr_type().into()];
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
			llvm_context_type,
		});

		Ok(())
	}

	pub fn define_closure(&mut self, closure: &SemanticClosure) -> Result<(), CodeGenError> {
		let closure_info = &self.closure_info[&closure.id];
		self.cur_fn = Some(closure_info.llvm_fn);
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
		for (i, param_id) in closure.param_ids.iter().enumerate() {
			let param_var = &self.program.variables[param_id];
			let llvm_param_val = closure_info.llvm_fn.get_nth_param((i + 1) as u32).unwrap();
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
                let prepared_stmt_gep = self.builder.build_struct_gep(
                    closure_info.llvm_context_type,
                    context_ptr,
                    closure.captured_variables.len() as u32,
                    "prepared_stmt_gep"
                )?;
                let prepared_stmt = self.builder.build_load(
                    self.ptr_type(),
                    prepared_stmt_gep,
                    "prepared_stmt"
                )?.into_pointer_value();

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

	pub fn gen_closure_instance(&mut self, closure_id: u32) -> Result<GenValue<'ctxt>, CodeGenError> {
		let closure = &self.program.closures[&closure_id];
		let llvm_context_type = self.closure_info[&closure_id].llvm_context_type;

		// Allocate context struct and populate captured variables
		let context_ptr = self.builder.build_malloc(llvm_context_type, "closure_context")?;
		for (i, (_, captured_id)) in closure.captured_variables.iter().enumerate() {
			let var_ptr = self.llvm_variables[&captured_id];
			let var_type = &self.program.variables[&captured_id].sem_type;
			let var_llvm_type = self.llvm_basic_type(var_type);
			let field_ptr = self.builder.build_struct_gep(
				llvm_context_type,
				context_ptr,
				i as u32,
				&format!("__captured__{}", self.program.variables[captured_id].name)
			)?;

			let var_value = self.builder.build_load(var_llvm_type, var_ptr, "load_captured")?;
			self.builder.build_store(field_ptr, var_value)?;
		}
        if let SemanticClosureBody::Query(ref query) = closure.body {
            // For query closures, prepare the statement and store it in the context
            let prepared_stmt = self.prepare_query(query)?;
            let prepared_stmt_gep = self.builder.build_struct_gep(
                llvm_context_type,
                context_ptr,
                closure.captured_variables.len() as u32,
                "prepared_stmt_gep"
            )?;
            self.builder.build_store(prepared_stmt_gep, prepared_stmt)?;
        }

		// Create callable struct instance
		let callable_ptr = self.builder.build_alloca(self.callable_struct_type, "callable")?;
		let fn_ptr = self.closure_info[&closure_id].llvm_fn.as_global_value().as_pointer_value();
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