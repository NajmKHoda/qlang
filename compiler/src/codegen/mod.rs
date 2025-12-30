use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{BasicTypeEnum, IntType, PointerType, VoidType};
use inkwell::basic_block::BasicBlock;
use inkwell::values::{AnyValue, GlobalValue, PointerValue};

use crate::tokens::{ProgramNode, StatementNode};

mod control_flow;
mod operations;
mod data;
mod string;
mod error;
mod function;
mod variable;
mod table;
mod array;
mod database;
mod runtime;

use variable::QLScope;
use variable::QLScopeType;
use function::QLFunction;
use table::QLTable;
use control_flow::QLLoop;
use runtime::RuntimeFunctions;
pub use error::CodeGenError;
pub use data::QLValue;
pub use data::QLType;
pub use operations::ComparisonOp;

pub struct CodeGen<'ctxt> {
    scopes: Vec<QLScope<'ctxt>>,
    functions: HashMap<String, QLFunction<'ctxt>>,
    tables: HashMap<String, QLTable<'ctxt>>,
    datasources: HashMap<String, PointerValue<'ctxt>>,
    runtime_functions: RuntimeFunctions<'ctxt>,
    loops: Vec<QLLoop<'ctxt>>,
    strings: HashMap<String, GlobalValue<'ctxt>>,

    cur_fn_name: Option<String>,
    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(mut self, program: &ProgramNode) -> Result<Module<'ctxt>, CodeGenError> {
        self.expose_runtime_function(self.runtime_functions.print_integer, QLType::Void, &[QLType::Integer]);
        self.expose_runtime_function(self.runtime_functions.print_boolean, QLType::Void, &[QLType::Bool]);
        self.expose_runtime_function(self.runtime_functions.print_string, QLType::Void, &[QLType::String]);
        self.expose_runtime_function(self.runtime_functions.input_integer, QLType::Integer, &[]);
        self.expose_runtime_function(self.runtime_functions.input_string, QLType::String, &[]);
        self.expose_runtime_function(self.runtime_functions.print_rc, QLType::Void, &[QLType::String]);

        self.gen_database_ptrs(&program.datasources);
        for table in &program.tables {
            self.gen_table(&table.name, &table.datasource_name, &table.columns)?;
        }

        for function in &program.functions {
            self.declare_user_function(&function.name, &function.return_type, &function.params)?;
            self.cur_fn_name = Some(function.name.to_string());

            let entry_block = self.append_block(format!("{}_entry", function.name).as_str());
            let function_terminates = self.gen_block_stmts(entry_block, &function.body, QLScopeType::FunctionScope)?;
            if !function_terminates {
                if function.return_type == QLType::Void {
                    self.builder.build_return(None)?;
                } else if function.name == "main" {
                    self.builder.build_return(Some(&self.int_type().const_zero()))?;
                } else {
                    return Err(CodeGenError::InexhaustiveReturnError(function.name.clone()));
                }
            }
        }
        
        let user_main_fn = self.functions.get("main").ok_or(CodeGenError::MissingMainError)?;
        let user_main_llvm_fn = user_main_fn.llvm_function;
        if user_main_fn.return_type != QLType::Integer || !user_main_fn.params.is_empty() {
            return Err(CodeGenError::BadMainSignatureError);
        }

        let main_fn_type = self.int_type().fn_type(
            &[self.int_type().into(), self.ptr_type().into()],
            false
        );
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let main_entry_block = self.context.append_basic_block(main_fn, "main_entry");
        self.builder.position_at_end(main_entry_block);

        let db_ptr_arr = self.init_databases(&program.datasources, main_fn)?;
        self.gen_const_strs()?;

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

    fn append_block(&mut self, name: &str) -> BasicBlock<'ctxt> {
        let cur_fn = self.cur_fn();
        self.context.append_basic_block(cur_fn.llvm_function, name)
    }

    fn gen_block_stmts(
        &mut self,
        block: BasicBlock<'ctxt>,
        stmts: &[StatementNode],
        scope_type: QLScopeType
    ) -> Result<bool, CodeGenError> {
        self.scopes.push(QLScope::new(scope_type));
        self.builder.position_at_end(block);

        let mut terminates = false;
        for stmt in stmts {
            terminates = stmt.gen_stmt(self)?;
            if terminates {
                break;
            }
        }

        if !terminates {
            self.release_scope(self.scopes.last().unwrap())?;
        }
        self.scopes.pop();
        Ok(terminates)
    }

    fn cur_fn(&self) -> &QLFunction<'ctxt> {
        let name = self.cur_fn_name.as_ref().unwrap();
        self.functions.get(name).unwrap()
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }
    fn bool_type(&self) -> IntType<'ctxt> { self.context.bool_type() }
    fn ptr_type(&self) -> PointerType<'ctxt> { self.context.ptr_type(Default::default()) }
    fn void_type(&self) -> VoidType<'ctxt> { self.context.void_type() }
    
    fn try_get_nonvoid_type(&self, ql_type: &QLType) -> Result<BasicTypeEnum<'ctxt>, CodeGenError> {
        match ql_type {
            QLType::Integer => Ok(self.int_type().into()),
            QLType::Bool => Ok(self.bool_type().into()),
            QLType::String => Ok(self.ptr_type().into()),
            QLType::Array(_) => Ok(self.ptr_type().into()),
            QLType::Table(table_name) => Ok(self.get_table(table_name)?.struct_type.into()),
            QLType::Void => Err(CodeGenError::UnexpectedTypeError)
        }
    }
}

pub fn gen_code(program: &ProgramNode) -> Result<(), CodeGenError> {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");

    let codegen = CodeGen {
        scopes: Vec::new(),
        functions: HashMap::new(),
        tables: HashMap::new(),
        datasources: HashMap::new(),
        runtime_functions: RuntimeFunctions::new(&context, &module),
        loops: Vec::new(),
        strings: HashMap::new(),
        cur_fn_name: None,
        context: &context,
        builder,
        module
    };

    let module = codegen.gen_code(program)?;

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

    let data_layout = target_machine.get_target_data().get_data_layout();
    module.set_triple(&target_triple);
    module.set_data_layout(&data_layout);

    let path = Path::new("out/main.o");
    target_machine.write_to_file(&module, FileType::Object, path)
        .map_err(|_| CodeGenError::TargetMachineWriteError)
}
