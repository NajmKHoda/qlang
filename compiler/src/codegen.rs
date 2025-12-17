use core::fmt;
use std::collections::HashMap;
use std::path::Path;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::{Builder, BuilderError};
use inkwell::support::LLVMString;
use inkwell::targets::{FileType, Target, TargetMachine};
use inkwell::types::{IntType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue, ValueKind, FunctionValue};
use inkwell::basic_block::BasicBlock;

use crate::tokens::{StatementNode};

pub enum CodeGenError {
    UnexpectedVoidError,
    UndefinedVariableError(String),

    BuilderError(BuilderError),
    ModuleVerificationError(LLVMString),
    TargetError(LLVMString),
    TargetMachineError,
    TargetMachineWriteError,
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::UnexpectedVoidError => write!(f, "Unexpected void return value"),
            CodeGenError::UndefinedVariableError(name) => write!(f, "Undefined variable: {name}"),
            CodeGenError::BuilderError(err) => write!(f, "Builder error: {err}"),
            CodeGenError::ModuleVerificationError(err) => write!(f, "Module verification error: {err}"),
            CodeGenError::TargetError(err) => write!(f, "Target error: {err}"),
            CodeGenError::TargetMachineError => write!(f, "Target machine creation error"),
            CodeGenError::TargetMachineWriteError => write!(f, "Target machine write to file error"),
        }
    }
}

impl From<BuilderError> for CodeGenError {
    fn from(err: BuilderError) -> Self { CodeGenError::BuilderError(err) }
}

#[derive(Clone, Copy, PartialEq)]
pub enum QLValue<'a> {
    Integer(IntValue<'a>),
    Void
}

impl<'a> TryFrom<BasicValueEnum<'a>> for QLValue<'a> {
    type Error = CodeGenError;

    fn try_from(value: BasicValueEnum<'a>) -> Result<Self, Self::Error> {
        match value {
            BasicValueEnum::IntValue(int_val) => Ok(QLValue::Integer(int_val)),
            _ => Err(CodeGenError::UnexpectedVoidError),
        }
    }
}

impl<'a> TryFrom<QLValue<'a>> for BasicValueEnum<'a> {
    type Error = CodeGenError;

    fn try_from(value: QLValue<'a>) -> Result<Self, Self::Error> {
        match value {
            QLValue::Integer(int_val) => Ok(BasicValueEnum::IntValue(int_val)),
            QLValue::Void => Err(CodeGenError::UnexpectedVoidError),
        }
    }
}

pub struct CodeGen<'ctxt> {
    vars: HashMap<String, PointerValue<'ctxt>>,
    cur_fn: Option<FunctionValue<'ctxt>>,
    context: &'ctxt Context,
    builder: Builder<'ctxt>,
    module: Module<'ctxt>
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(mut self, stmts: &Vec<StatementNode>) -> Result<Module<'ctxt>, CodeGenError> {
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

    fn gen_stmts(&mut self, stmts: &Vec<StatementNode>) -> Result<(), CodeGenError> {
        for stmt in stmts {
            stmt.gen_stmt(self)?;
        }
        Ok(())
    }

    fn int_type(&self) -> IntType<'ctxt> { self.context.i32_type() }

    pub fn const_int(&self, value: i32) -> QLValue<'ctxt> {
        QLValue::Integer(self.int_type().const_int(value as u64, false))
    }

    pub fn load_var(&self, name: &String) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let Some(pointer) = self.vars.get(name) {
            let res = self.builder.build_load(self.int_type(), *pointer, "load").map(|v| v.into_int_value())?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UndefinedVariableError(name.clone()))
        }
    }

    pub fn store_var(&mut self, name: &String, value: QLValue<'ctxt>) -> Result<(), CodeGenError> {
        if !self.vars.contains_key(name) {
            // Allocate memory on first assignment
            let pointer = self.builder.build_alloca(self.int_type(), name)?;
            self.vars.insert(name.clone(), pointer);
        }
        
        let pointer = self.vars[name];
        self.builder.build_store::<BasicValueEnum>(pointer, value.try_into()?)?;
        Ok(())
    } 
    
    pub fn add(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_add(int1, int2, "sum")?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UnexpectedVoidError)
        }
    }

    pub fn call(&self, fn_name: &str, args: Vec<QLValue<'ctxt>>) -> Result<QLValue<'ctxt>, CodeGenError> {
        let function = self.module.get_function(fn_name)
            .ok_or_else(|| CodeGenError::UndefinedVariableError(fn_name.to_string()))?;
        let arg_values: Vec<BasicMetadataValueEnum> = args
            .into_iter()
            .map(|v| BasicValueEnum::try_from(v).map(BasicMetadataValueEnum::from))
            .collect::<Result<Vec<BasicMetadataValueEnum>, CodeGenError>>()?;
        let call_site = self.builder.build_call(function, &arg_values, "call")?;
        match call_site.try_as_basic_value() {
            ValueKind::Basic(value) => Ok(value.try_into()?),
            ValueKind::Instruction(_) => Ok(QLValue::Void),
        }
    }

    pub fn gen_conditional(
        &mut self,
        conditional: QLValue<'ctxt>,
        then_stmts: &Vec<StatementNode>,
        else_stmts: &Vec<StatementNode>
    ) -> Result<(), CodeGenError> {
        if let QLValue::Integer(cond_int32) = conditional {
            let then_block = self.append_block("then");
            let else_block = self.append_block("else");
            let merge_block = self.append_block("merge");
            
            let zero = self.int_type().const_int(0, false);
            let cond = self.builder.build_int_compare(inkwell::IntPredicate::NE, cond_int32, zero, "cond")?;

            self.builder.build_conditional_branch(cond, then_block, else_block)?;
            self.builder.position_at_end(then_block);
            self.gen_stmts(then_stmts)?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(else_block);
            self.gen_stmts(else_stmts)?;
            self.builder.build_unconditional_branch(merge_block)?;

            self.builder.position_at_end(merge_block);

            Ok(())
        } else {
            Err(CodeGenError::UnexpectedVoidError)
        }
    }
}

pub fn gen_code(stmts: &Vec<StatementNode>) -> Result<(), CodeGenError> {
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



