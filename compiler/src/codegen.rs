use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::{Builder, BuilderError};
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, IntValue, PointerValue};

use crate::tokens::{ExpressionNode, StatementNode};

struct CodeGen<'ctxt> {
    vars: HashMap<String, PointerValue<'ctxt>>,
    context: &'ctxt Context,
    module: Module<'ctxt>,
    builder: Builder<'ctxt>,

    int_type: IntType<'ctxt>,
    print_fn: FunctionValue<'ctxt>
}

impl<'ctxt> CodeGen<'ctxt> {
    fn gen_code(&mut self, stmts: &Vec<StatementNode>) -> Result<(), BuilderError> {
        let main_type = self.int_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);

        let basic_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(basic_block);

        for stmt in stmts {
            match stmt {
                StatementNode::Assignment(_,_) => { self.gen_assignment(stmt)?; }
                StatementNode::Print(_) => { self.gen_print(stmt)?; }
            }
        }

        self.builder.build_return(Some(&self.int_type.const_int(0, false)))?;
        
        let res = self.module.verify();
        match res {
            Ok(_) => {
                let _ = self.module.print_to_file("out.debug");
                Ok(())
            }
            Err(msg) => panic!("Building failed: {msg}")
        }
    }

    fn gen_assignment(&mut self, node: &StatementNode) -> Result<(), BuilderError> {
        if let StatementNode::Assignment(var_name, expr) = node {
            if !self.vars.contains_key(var_name) {
                // Allocate memory on first assignment
                let pointer = self.builder.build_alloca(self.int_type, var_name)?;
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

    fn gen_print(&self, node: &StatementNode) -> Result<(), BuilderError> {
        if let StatementNode::Print(expr) = node {
            let value = self.gen_expr(expr)?;
            self.builder.build_call(self.print_fn, &[value.into()], "printi")?;
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn gen_expr(&self, node: &ExpressionNode) -> Result<IntValue<'ctxt>, BuilderError> {
        match node {
            ExpressionNode::Integer(x) => Ok(self.int_type.const_int(*x as u64, false)),
            ExpressionNode::QName(name) => {
                if let Some(pointer) = self.vars.get(name) {
                    self.builder.build_load(self.int_type, *pointer, "assign").map(|v| v.into_int_value())
                } else {
                    panic!("Undefined variable: {name}")
                }
            },
            ExpressionNode::Add(expr1, expr2) => {
                let val1 = self.gen_expr(expr1)?;
                let val2 = self.gen_expr(expr2)?;
                self.builder.build_int_add(val1, val2, "sum")
            }
        }
    }
}

pub fn gen_code(stmts: &Vec<StatementNode>) -> Result<(), BuilderError> {
    let context = Context::create();
    let module = context.create_module("db-lang");
    let builder = context.create_builder();

    let i32_type = context.i32_type();

    let void_type = context.void_type();
    let print_type = void_type.fn_type(&[i32_type.into()], false);
    let print_fn = module.add_function("printi", print_type, None);

    let mut codegen = CodeGen {
        vars: HashMap::<String, PointerValue>::new(),
        context: &context,
        module,
        builder,
        int_type: i32_type,
        print_fn
    };

    codegen.gen_code(stmts)?;
    Ok(())
}



