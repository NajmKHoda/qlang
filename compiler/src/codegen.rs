use core::fmt;
use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::{Builder, BuilderError};
use inkwell::support::LLVMString;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{IntType, VoidType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};

use crate::tokens::{ExpressionNode, StatementNode};

pub enum CodeGenError {
    BuilderError(BuilderError),
    UndefinedVariableError(String),
    ModuleVerificationError(LLVMString),
    TargetError(LLVMString),
    TargetMachineError(),
    TargetMachineWriteError(),
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::BuilderError(err) => write!(f, "Builder error: {err}"),
            CodeGenError::UndefinedVariableError(name) => write!(f, "Undefined variable: {name}"),
            CodeGenError::ModuleVerificationError(err) => write!(f, "Module verification error: {err}"),
            CodeGenError::TargetError(err) => write!(f, "Target error: {err}"),
            CodeGenError::TargetMachineError() => write!(f, "Target machine creation error"),
            CodeGenError::TargetMachineWriteError() => write!(f, "Target machine write to file error"),
        }
    }
}

impl From<BuilderError> for CodeGenError {
    fn from(err: BuilderError) -> Self { CodeGenError::BuilderError(err) }
}

struct CodeGen<'ctxt> {
    vars: HashMap<String, PointerValue<'ctxt>>,
    context: &'ctxt Context,
    builder: Builder<'ctxt>,
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(&mut self, stmts: &Vec<StatementNode>) -> Result<Module<'ctxt>, CodeGenError> {
        let module = self.context.create_module("db-lang");

        let main_type = self.int_type().fn_type(&[], false);
        let main_fn = module.add_function("main", main_type, None);

        let print_type = self.void_type().fn_type(&[self.int_type().into()], false);
        let print_fn = module.add_function("printi", print_type, None);

        let basic_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(basic_block);

        for stmt in stmts {
            match stmt {
                StatementNode::Assignment(_,_) => self.gen_assignment(stmt)?,
                StatementNode::Print(_) => self.gen_print(stmt, print_fn)?
            }
        }

        self.builder.build_return(Some(&self.int_type().const_int(0, false)))?;

        module.verify().map_err(|e| CodeGenError::ModuleVerificationError(e))?;
        Ok(module)
    }

    fn gen_assignment(&mut self, node: &StatementNode) -> Result<(), CodeGenError> {
        if let StatementNode::Assignment(var_name, expr) = node {
            if !self.vars.contains_key(var_name) {
                // Allocate memory on first assignment
                let pointer = self.builder.build_alloca(self.int_type(), var_name)?;
                self.vars.insert(var_name.clone(), pointer);
            }
            
            let pointer = self.vars[var_name];
            let value = self.gen_expr(expr)?;
            self.builder.build_store(pointer, value)?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn gen_print(&self, node: &StatementNode, print_fn: FunctionValue) -> Result<(), CodeGenError> {
        if let StatementNode::Print(expr) = node {
            let value = self.gen_expr(expr)?;
            self.builder.build_call(print_fn, &[value.into()], "printi")?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn gen_expr(&self, node: &ExpressionNode) -> Result<IntValue<'ctxt>, CodeGenError> {
        match node {
            ExpressionNode::Integer(x) => Ok(self.int_type().const_int(*x as u64, false)),
            ExpressionNode::QName(name) => {
                if let Some(pointer) = self.vars.get(name) {
                    let res = self.builder.build_load(self.int_type(), *pointer, "assign").map(|v| v.into_int_value())?;
                    Ok(res)
                } else {
                    Err(CodeGenError::UndefinedVariableError(name.clone()))
                }
            },
            ExpressionNode::Add(expr1, expr2) => {
                let val1 = self.gen_expr(expr1)?;
                let val2 = self.gen_expr(expr2)?;
                Ok(self.builder.build_int_add(val1, val2, "sum")?)
            }
        }
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }
    fn void_type(&self) -> VoidType<'ctxt> { self.context.void_type() }
}

pub fn gen_code(stmts: &Vec<StatementNode>) -> Result<(), CodeGenError> {
    let context = Context::create();
    let builder = context.create_builder();
    
    let mut codegen = CodeGen {
        vars: HashMap::<String, PointerValue>::new(),
        context: &context,
        builder,
    };

    let module = codegen.gen_code(stmts)?;
  
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
    ).ok_or_else(|| CodeGenError::TargetMachineError())?;

    let data_layout = target_machine.get_target_data().get_data_layout();
    module.set_triple(&target_triple);
    module.set_data_layout(&data_layout);

    let path = Path::new("out/main.o");
    target_machine.write_to_file(&module, FileType::Object, path)
        .map_err(|_| CodeGenError::TargetMachineWriteError())
}



