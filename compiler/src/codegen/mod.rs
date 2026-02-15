use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{FileType, Target, TargetData, TargetMachine};
use inkwell::types::{IntType, PointerType, VoidType};
use inkwell::values::{AnyValue, FunctionValue, GlobalValue, PointerValue};

use crate::semantics::{SemanticExpression, SemanticExpressionKind, SemanticProgram, SemanticStatement, SemanticTypeKind};

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
use closure::GenClosureInfo;
use runtime::Runtime;
pub use error::CodeGenError;

pub struct CodeGen<'ctxt> {
    program: &'ctxt SemanticProgram,

    datasource_ptrs: HashMap<u32, PointerValue<'ctxt>>,
    llvm_functions: HashMap<u32, FunctionValue<'ctxt>>,
    llvm_variables: HashMap<u32, PointerValue<'ctxt>>,
    table_info: HashMap<u32, GenTableInfo<'ctxt>>,
    struct_info: HashMap<u32, GenStructInfo<'ctxt>>,
    loop_info: HashMap<u32, GenLoopInfo<'ctxt>>,
    closure_info: HashMap<u32, GenClosureInfo<'ctxt>>,
    runtime: Runtime<'ctxt>,
    strings: HashMap<String, GlobalValue<'ctxt>>,

    cur_fn: Option<FunctionValue<'ctxt>>,
    vars_to_drop: Vec<u32>,

    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>,
    target_data: TargetData,
}

impl<'ctxt> CodeGen<'ctxt> {
    fn _gen_code(mut self) -> Result<Module<'ctxt>, CodeGenError> {
        for datasource in self.program.datasources.values() {
            self.gen_database_ptr(&datasource);
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
        
        let main_fn_type = self.int_type().fn_type(
            &[self.int_type().into(), self.ptr_type().into()],
            false
        );
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let main_entry_block = self.context.append_basic_block(main_fn, "main_entry");
        self.builder.position_at_end(main_entry_block);

        let db_ptr_arr = self.init_databases(main_fn)?;
        self.gen_const_strs()?;

        let user_main_llvm_fn = self.module.get_function("__ql__user_main").unwrap();
        let call_site = self.builder.build_call(
            user_main_llvm_fn,
            &[],
            "call_user_main"
        )?.as_any_value_enum().into_int_value();

        self.drop_const_strs()?;
        self.close_databases(db_ptr_arr)?;

        self.builder.build_return(Some(&call_site))?;

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
                self.remove_if_owned(value)
            }
            SemanticStatement::Conditional { branches, else_branch } => {
                self.gen_conditional(branches, else_branch)
            }
            SemanticStatement::ConditionalLoop { condition, body, id } => {
                self.gen_loop(condition, body, *id)
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
                self.vars_to_drop.push(*variable_id);
                Ok(())
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
            SemanticExpressionKind::Add { left, right } => {
                let val1 = self.gen_eval(&left)?;
                let val2 = self.gen_eval(&right)?;
                self.gen_add(val1, val2)
            }
            SemanticExpressionKind::Subtract { left, right } => {
                let val1 = self.gen_eval(&left)?;
                let val2 = self.gen_eval(&right)?;
                self.gen_subtract(val1, val2)
            }
            SemanticExpressionKind::Compare { left, right, op } => {
                let val1 = self.gen_eval(&left)?;
                let val2 = self.gen_eval(&right)?;
                self.gen_compare(val1, val2, *op)
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
            datasource_ptrs: HashMap::new(),
            llvm_variables: HashMap::new(),
            llvm_functions: HashMap::new(),
            table_info: HashMap::new(),
            struct_info: HashMap::new(),
            loop_info: HashMap::new(),
            closure_info: HashMap::new(),
            runtime: Runtime::new(&context, &module),
            strings: HashMap::new(),
            cur_fn: None,
            vars_to_drop: vec![],
            context: &context,
            builder,
            module,
            target_data,
        };

        let module = codegen._gen_code()?;
        let path = Path::new("out/main.o");
        target_machine.write_to_file(&module, FileType::Object, path)
            .map_err(|_| CodeGenError::TargetMachineWriteError)
    }
}
