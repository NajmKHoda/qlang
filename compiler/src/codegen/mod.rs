use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{IntType, PointerType, StructType, VoidType};
use inkwell::values::{AnyValue, FunctionValue, GlobalValue, PointerValue};

use crate::semantics::{SemanticExpression, SemanticExpressionKind, SemanticProgram, SemanticStatement, SemanticType, SemanticTypeKind};

mod control_flow;
mod operations;
mod data;
mod string;
mod error;
mod function;
mod closure;
mod variable;
mod table;
mod array;
mod database;
mod runtime;
mod structs;

use data::GenValue;
use table::GenTableInfo;
use structs::GenStructInfo;
use control_flow::GenLoopInfo;
use control_flow::GenTransactionInfo;
use closure::GenClosureInfo;
use runtime::Runtime;
pub use error::CodeGenError;

pub struct CodeGen<'ctxt> {
    program: &'ctxt SemanticProgram,

    llvm_functions: HashMap<u32, FunctionValue<'ctxt>>,
    llvm_variables: HashMap<u32, PointerValue<'ctxt>>,
    table_info: HashMap<u32, GenTableInfo<'ctxt>>,
    struct_info: HashMap<u32, GenStructInfo<'ctxt>>,
    loop_info: HashMap<u32, GenLoopInfo<'ctxt>>,
    closure_info: HashMap<u32, GenClosureInfo<'ctxt>>,
    runtime: Runtime<'ctxt>,
    strings: HashMap<String, GlobalValue<'ctxt>>,

    cur_fn: Option<FunctionValue<'ctxt>>,
    cur_function_id: Option<u32>,
    cur_closure_id: Option<u32>,
    transaction_stack: Vec<GenTransactionInfo<'ctxt>>,

    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>,
}

impl<'ctxt> CodeGen<'ctxt> {
    fn _gen_code(mut self) -> Result<Module<'ctxt>, CodeGenError> {
        for _struct in self.program.structs.values() {
            self.gen_struct(_struct)?;
        }
        for table in self.program.tables.values() {
            self.gen_table(&table)?;
        }

        // Forward-declare closures and functions
        for closure in self.program.closures.values() {
            self.declare_closure(closure)?;
        }
        for function in self.program.functions.values() {
            self.declare_function(function)?;
        }

        // Now, define closures and functions
        for closure in self.program.closures.values() {
            self.define_closure(closure)?;
        }
        for function in self.program.functions.values() {
            self.define_function(&function)?;
        }

        // Generate main function that calls user_main
        let main_fn_type = self.int_type().fn_type(
            &[self.int_type().into(), self.ptr_type().into()],
            false
        );
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let main_entry_block = self.context.append_basic_block(main_fn, "main_entry");
        self.builder.position_at_end(main_entry_block);

        // Setup (initialize databases and constant strings)
        let argc = main_fn.get_nth_param(0).unwrap().into_int_value();
        let argv = main_fn.get_nth_param(1).unwrap().into_pointer_value();
        self.builder.build_call(self.runtime.init_dbs, &[
            argc.into(),
            argv.into(),
            self.int_type().const_int(self.program.datasources.len() as u64, false).into()
        ], "init_dbs")?;
        self.gen_const_strs()?;

        // Call user_main
        let user_main_llvm_fn = self.module.get_function("__ql__user_main").unwrap();
        let user_main_sem_fn = self.program.functions.values()
            .find(|f| f.name == "main")
            .unwrap();
        let call_site = self.builder.build_call(user_main_llvm_fn, &[], "call_user_main")?;
        let main_ret_val = if user_main_sem_fn.is_failable {
            let result_struct = call_site.as_any_value_enum().into_struct_value();
            let is_error = self.builder.build_extract_value(result_struct, 0, "main_is_error")?
                .into_int_value();
            let ok_block = self.context.append_basic_block(main_fn, "main_ok");
            let err_block = self.context.append_basic_block(main_fn, "main_err");
            let merge_block = self.context.append_basic_block(main_fn, "main_ret_merge");
            let ret_ptr = self.builder.build_alloca(self.int_type(), "main_ret_ptr")?;

            self.builder.build_conditional_branch(is_error, err_block, ok_block)?;

            self.builder.position_at_end(err_block);
            self.builder.build_store(ret_ptr, self.int_type().const_int(1, false))?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(ok_block);
            let ok_val = self.builder.build_extract_value(result_struct, 1, "main_ok_val")?
                .into_int_value();
            self.builder.build_store(ret_ptr, ok_val)?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(merge_block);
            self.builder.build_load(self.int_type(), ret_ptr, "main_ret")?.into_int_value()
        } else {
            call_site.as_any_value_enum().into_int_value()
        };

        // Teardown (drop constant strings and close databases)
        self.drop_const_strs()?;
        self.builder.build_call(self.runtime.close_dbs, &[], "close_dbs")?;
        self.builder.build_return(Some(&main_ret_val))?;

        if let Err(msg) = self.module.print_to_file("out/main.debug") {
            eprintln!("Failed to write debug LLVM IR: {}", msg);
        }

        self.module.verify().map_err(|e| CodeGenError::ModuleVerificationError(e))?;
        Ok(self.module)
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }
    fn bool_type(&self) -> IntType<'ctxt> { self.context.bool_type() }
    fn ptr_type(&self) -> PointerType<'ctxt> { self.context.ptr_type(Default::default()) }
    fn void_type(&self) -> VoidType<'ctxt> { self.context.void_type() }

    fn llvm_result_type(&self, sem_type: &SemanticType) -> StructType<'ctxt> {
        match sem_type.kind() {
            SemanticTypeKind::Void => self.context.struct_type(&[self.bool_type().into()], false),
            _ => self.context.struct_type(&[
                self.bool_type().into(),
                self.llvm_basic_type(sem_type).into(),
            ], false),
        }
    }

    fn cur_executable_return_type(&self) -> SemanticType {
        if let Some(function_id) = self.cur_function_id {
            self.program.functions[&function_id].return_type.clone()
        } else if let Some(closure_id) = self.cur_closure_id {
            self.program.closures[&closure_id].return_type.clone()
        } else {
            SemanticType::new(SemanticTypeKind::Void)
        }
    }

    fn cur_executable_is_failable(&self) -> bool {
        if let Some(function_id) = self.cur_function_id {
            self.program.functions[&function_id].is_failable
        } else if let Some(closure_id) = self.cur_closure_id {
            self.program.closures[&closure_id].is_failable
        } else {
            false
        }
    }

    fn gen_stmt(&mut self, stmt: &SemanticStatement) -> Result<(), CodeGenError> {
        match &stmt {
            SemanticStatement::VariableDeclaration { variable_id, init_expr } => {
                let init_value = self.gen_eval(init_expr)?;
                self.define_var(*variable_id, init_value)
            }
            SemanticStatement::VariableAssignment { variable_id, expr } => {
                let value = self.gen_eval(expr)?;
                self.store_var(*variable_id, value)
            }
            SemanticStatement::LoneExpression(expr) => {
                let value = self.gen_eval(expr)?;
                self.remove_if_owned(value, &expr.sem_type)
            }
            SemanticStatement::Conditional { branches, else_branch } => {
                self.gen_conditional(branches, else_branch)
            }
            SemanticStatement::ConditionalLoop { condition, body, id } => {
                self.gen_loop(condition, body, *id)
            }
            SemanticStatement::Transaction { body, rollback_body, id } => {
                self.gen_transaction(body, rollback_body, *id)
            }
            SemanticStatement::Return(expr) => {
                self.gen_return(expr)
            }
            SemanticStatement::Break(loop_id) => {
                self.gen_break(*loop_id)
            }
            SemanticStatement::Continue(loop_id) => {
                self.gen_continue(*loop_id)
            }
            SemanticStatement::DropVariable(variable_id) => {
                self.drop_var(*variable_id)
            }
        }
    }

    fn gen_eval(&mut self, expr: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let expr_type_kind = expr.sem_type.kind();
        match &expr.kind {
            SemanticExpressionKind::IntegerLiteral(value) => {
                Ok(GenValue::Integer(self.int_type().const_int(*value as u64, false)))
            },
            SemanticExpressionKind::BoolLiteral(value) => {
                Ok(GenValue::Bool(self.bool_type().const_int(*value as u64, false)))
            },
            SemanticExpressionKind::StringLiteral(value) => {
                self.const_str(&value)
            },
            SemanticExpressionKind::Struct(fields) => {
                let SemanticTypeKind::NamedStruct(struct_id, _) = expr_type_kind else {
                    panic!("Expected NamedStruct type")
                };
                self.gen_struct_value(struct_id, fields)
            }
            SemanticExpressionKind::Array(elements) => {
                let SemanticTypeKind::Array(elem_type) = expr_type_kind else {
                    panic!("Expected Array type")
                };
                self.gen_array(elements, &elem_type)
            }
            SemanticExpressionKind::Closure(closure_id) => {
                self.gen_callable(*closure_id)
            }
            SemanticExpressionKind::Variable(var_id) => {
                self.load_var(*var_id)
            }
            SemanticExpressionKind::StructField { struct_expr, index } => {
                let struct_value = self.gen_eval(struct_expr)?;
                self.get_field_value(struct_value, *index)
            }
            SemanticExpressionKind::ArrayIndex { array_expr, index_expr } => {
                let array_value = self.gen_eval(array_expr)?;
                let index_value = self.gen_eval(index_expr)?;
                self.gen_array_index(array_value, index_value)
            }
            SemanticExpressionKind::Range { start, end, inclusive, step } => {
                self.gen_range(start.as_deref(), end.as_deref(), *inclusive, step.as_deref())
            }
            SemanticExpressionKind::Add { left, right } => {
                self.gen_add(left, right)
            }
            SemanticExpressionKind::Subtract { left, right } => {
                self.gen_subtract(left, right)
            }
            SemanticExpressionKind::Compare { left, right, op } => {
                self.gen_compare(left, right, *op)
            }
            SemanticExpressionKind::DirectFunctionCall { function_id, args } => {
                self.gen_direct_call(*function_id, args)
            }
            SemanticExpressionKind::IndirectFunctionCall { function_expr, args } => {
                self.gen_indirect_call(function_expr, args)
            }
            SemanticExpressionKind::BuiltinFunctionCall { function, args } => {
                self.gen_builtin_call(*function, args)
            }
            SemanticExpressionKind::BuiltinMethodCall { receiver, method, args } => {
                let receiver_val = self.gen_eval(receiver)?;
                self.gen_method_call(receiver_val, *method, args)
            }
            SemanticExpressionKind::ImmediateQuery(query) => {
                self.gen_immediate_query(query)
            }
        }
    }

    pub fn gen_code(program: &SemanticProgram) -> Result<(), CodeGenError> {
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("main");

        Target::initialize_all(&Default::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(|e| CodeGenError::TargetError(e))?;
        let target_machine = target.create_target_machine(
            &target_triple,
            "generic",
            "",
            inkwell::OptimizationLevel::Default,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        ).ok_or_else(|| CodeGenError::TargetMachineError)?;
        let target_data = target_machine.get_target_data();
        let data_layout = target_data.get_data_layout();
        module.set_triple(&target_triple);
        module.set_data_layout(&data_layout);

        let codegen = CodeGen {
            program,
            llvm_variables: HashMap::new(),
            llvm_functions: HashMap::new(),
            table_info: HashMap::new(),
            struct_info: HashMap::new(),
            loop_info: HashMap::new(),
            closure_info: HashMap::new(),
            runtime: Runtime::new(&context, &module),
            strings: HashMap::new(),
            cur_fn: None,
            cur_function_id: None,
            cur_closure_id: None,
            transaction_stack: vec![],
            context: &context,
            builder,
            module,
        };

        let module = codegen._gen_code()?;
        let path = Path::new("out/main.o");
        target_machine.write_to_file(&module, FileType::Object, path)
            .map_err(|_| CodeGenError::TargetMachineWriteError)
    }
}
