use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{IntType};
use inkwell::values::{PointerValue, FunctionValue};
use inkwell::basic_block::BasicBlock;

use crate::tokens::{StatementNode};

mod control_flow;
mod operations;
mod data;
mod error;
pub use error::CodeGenError;
pub use data::QLValue;
pub use operations::ComparisonOp;

pub struct CodeGen<'ctxt> {
    vars: HashMap<String, PointerValue<'ctxt>>,
    cur_fn: Option<FunctionValue<'ctxt>>,
    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(mut self, stmts: Vec<StatementNode>) -> Result<Module<'ctxt>, CodeGenError> {
        let main_type = self.int_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        self.cur_fn = Some(main_fn);

        let block = self.append_block("entry");
        self.builder.position_at_end(block);
        self.gen_stmts(stmts)?;
        self.builder.build_return(Some(&self.int_type().const_int(0, false)))?;

        self.module.verify().map_err(|e| CodeGenError::ModuleVerificationError(e))?;
        Ok(self.module)
    }

    fn append_block(&mut self, name: &str) -> BasicBlock<'ctxt> {
        let cur_fn = self.cur_fn.unwrap();
        self.context.append_basic_block(cur_fn, name)
    }

    fn gen_stmts(&mut self, stmts: Vec<StatementNode>) -> Result<(), CodeGenError> {
        for stmt in stmts {
            stmt.gen_stmt(self)?;
        }
        Ok(())
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }
}

pub fn gen_code(stmts: Vec<StatementNode>) -> Result<(), CodeGenError> {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");

    let print_type = context.void_type().fn_type(&[context.i32_type().into()], false);
    module.add_function("printi", print_type, None);

    let input_type = context.i32_type().fn_type(&[], false);
    module.add_function("inputi", input_type, None);
    
    let codegen = CodeGen {
        vars: HashMap::<String, PointerValue>::new(),
        cur_fn: None,
        context: &context,
        builder,
        module
    };

    let module = codegen.gen_code(stmts)?;
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
