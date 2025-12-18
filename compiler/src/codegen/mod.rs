use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{BasicTypeEnum, IntType, VoidType};
use inkwell::basic_block::BasicBlock;

use crate::codegen::control_flow::QLFunction;
use crate::tokens::{ProgramNode, StatementNode};

mod control_flow;
mod operations;
mod data;
mod error;

use data::QLVariable;
pub use error::CodeGenError;
pub use data::QLValue;
pub use data::QLType;
pub use operations::ComparisonOp;

pub struct CodeGen<'ctxt> {
    vars: HashMap<String, QLVariable<'ctxt>>,
    functions: HashMap<String, QLFunction<'ctxt>>,
    cur_fn_name: Option<String>,
    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(mut self, program: ProgramNode) -> Result<Module<'ctxt>, CodeGenError> {
        self.declare_extern_function("printi", QLType::Void, &[QLType::Integer])?;
        self.declare_extern_function("printb", QLType::Void, &[QLType::Bool])?;
        self.declare_extern_function("inputi", QLType::Integer, &[])?;

        for function in program.functions {
            self.declare_function(&function.name, function.return_type, function.params)?;
            self.cur_fn_name = Some(function.name.to_string());

            let block = self.append_block("entry");
            self.builder.position_at_end(block);
            self.gen_stmts(function.body)?;
        }

        self.module.verify().map_err(|e| CodeGenError::ModuleVerificationError(e))?;
        Ok(self.module)
    }

    fn append_block(&mut self, name: &str) -> BasicBlock<'ctxt> {
        let cur_fn = self.cur_fn();
        self.context.append_basic_block(cur_fn.llvm_function, name)
    }

    fn gen_stmts(&mut self, stmts: Vec<StatementNode>) -> Result<(), CodeGenError> {
        for stmt in stmts {
            stmt.gen_stmt(self)?;
        }
        Ok(())
    }

    fn cur_fn(&self) -> &QLFunction<'ctxt> {
        let name = self.cur_fn_name.as_ref().unwrap();
        self.functions.get(name).unwrap()
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }
    fn bool_type(&self) -> IntType<'ctxt> { self.context.bool_type() }
    fn void_type(&self) -> VoidType<'ctxt> { self.context.void_type() }
    
    fn try_get_nonvoid_type(&self, ql_type: &QLType) -> Result<BasicTypeEnum<'ctxt>, CodeGenError> {
        match ql_type {
            QLType::Integer => Ok(self.int_type().into()),
            QLType::Bool => Ok(self.bool_type().into()),
            QLType::Void => Err(CodeGenError::UnexpectedTypeError)
        }
    }
}

pub fn gen_code(program: ProgramNode) -> Result<(), CodeGenError> {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");
    
    let codegen = CodeGen {
        vars: HashMap::<String, QLVariable>::new(),
        functions: HashMap::<String, QLFunction>::new(),
        cur_fn_name: None,
        context: &context,
        builder,
        module
    };

    let module = codegen.gen_code(program)?;
    if let Err(msg) = module.print_to_file("out/main.debug") {
        eprintln!("Failed to write debug LLVM IR: {}", msg);
    }

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
