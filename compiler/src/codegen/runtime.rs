use inkwell::types::{FunctionType};
use inkwell::{context::Context};
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use super::function::QLParameter;
use super::{CodeGen, QLFunction, QLType};

#[derive(Clone, Copy)]
pub(super) struct RuntimeFunction<'ctxt> {
    name: &'static str,
    llvm_function: FunctionValue<'ctxt>,
}

impl<'ctxt> From<RuntimeFunction<'ctxt>> for FunctionValue<'ctxt> {
    fn from(rtf: RuntimeFunction<'ctxt>) -> Self {
        rtf.llvm_function
    }
}

pub(super) struct RuntimeFunctions<'ctxt> {
    pub(super) print_integer: RuntimeFunction<'ctxt>,
    pub(super) print_boolean: RuntimeFunction<'ctxt>,
    pub(super) print_string: RuntimeFunction<'ctxt>,
    pub(super) input_integer: RuntimeFunction<'ctxt>,
    pub(super) input_string: RuntimeFunction<'ctxt>,

    pub(super) new_string: RuntimeFunction<'ctxt>,
    pub(super) add_string_ref: RuntimeFunction<'ctxt>,
    pub(super) remove_string_ref: RuntimeFunction<'ctxt>,
    pub(super) concat_string: RuntimeFunction<'ctxt>,
    pub(super) compare_string: RuntimeFunction<'ctxt>,

    pub(super) print_rc: RuntimeFunction<'ctxt>,
}

impl<'ctxt> RuntimeFunctions<'ctxt> {
    fn add_runtime_function(
        module: &Module<'ctxt>,
        name: &'static str,
        function_type: FunctionType<'ctxt>,
    ) -> RuntimeFunction<'ctxt> {
        RuntimeFunction {
            name,
            llvm_function: module.add_function(name, function_type, None)
        }
    }

    pub(super) fn new(context: &'ctxt Context, module: &Module<'ctxt>) -> Self {
        let void_type = context.void_type();
        let int_type = context.i32_type();
        let bool_type = context.bool_type();
        let ptr_type = context.ptr_type(Default::default());

        let print_integer = Self::add_runtime_function(
            module,
            "printi",
            void_type.fn_type(&[int_type.into()], false),
        );

        let print_boolean = Self::add_runtime_function(
            module,
            "printb",
            void_type.fn_type(&[bool_type.into()], false),
        );

        let print_string = Self::add_runtime_function(
            module,
            "prints",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let input_integer = Self::add_runtime_function(
            module,
            "inputi",
            int_type.fn_type(&[], false),
        );

        let input_string = Self::add_runtime_function(
            module,
            "inputs",
            ptr_type.fn_type(&[], false),
        );

        let new_string = Self::add_runtime_function(
            module,
            "__ql__QLString_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into(), bool_type.into()], false),
        );

        let add_string_ref = Self::add_runtime_function(
            module,
            "__ql__QLString_add_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let remove_string_ref = Self::add_runtime_function(
            module,
            "__ql__QLString_remove_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let concat_string = Self::add_runtime_function(
            module,
            "__ql__QLString_concat",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let compare_string = Self::add_runtime_function(
            module,
            "__ql__QLString_compare",
            int_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let print_rc = Self::add_runtime_function(
            module,
            "_print_rc",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        RuntimeFunctions {
            print_integer,
            print_boolean,
            print_string,
            input_integer,
            input_string,
            new_string,
            add_string_ref,
            remove_string_ref,
            concat_string,
            compare_string,
            print_rc,
        }
    }
}


impl<'ctxt> CodeGen<'ctxt> {
    pub(super) fn expose_runtime_function(
        &mut self,
        runtime_function: RuntimeFunction<'ctxt>,
        return_type: QLType,
        param_types: &[QLType],
    ) {
        self.functions.insert(runtime_function.name.to_string(), QLFunction {
            name: runtime_function.name.to_string(),
            llvm_function: runtime_function.llvm_function,
            return_type,
            params: param_types.iter().enumerate().map(|(i, t)| QLParameter {
                name: format!("arg{}", i),
                ql_type: t.clone(),
            }).collect(),
        });
    }
}