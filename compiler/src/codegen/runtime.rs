use inkwell::AddressSpace;
use inkwell::types::{FunctionType, StructType};
use inkwell::{context::Context};
use inkwell::module::{Linkage, Module};
use inkwell::values::{FunctionValue, GlobalValue};

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
    pub(super) type_info_type: StructType<'ctxt>,
    pub(super) int_type_info: GlobalValue<'ctxt>,
    pub(super) bool_type_info: GlobalValue<'ctxt>,
    pub(super) string_type_info: GlobalValue<'ctxt>,
    pub(super) array_type_info: GlobalValue<'ctxt>,

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

    pub(super) new_array: RuntimeFunction<'ctxt>,
    pub(super) add_array_ref: RuntimeFunction<'ctxt>,
    pub(super) remove_array_ref: RuntimeFunction<'ctxt>,
    pub(super) index_array: RuntimeFunction<'ctxt>,
    pub(super) append_array: RuntimeFunction<'ctxt>,
    pub(super) array_length: RuntimeFunction<'ctxt>,
    pub(super) pop_array: RuntimeFunction<'ctxt>,

    pub(super) init_dbs: RuntimeFunction<'ctxt>,
    pub(super) close_dbs: RuntimeFunction<'ctxt>,
    
    pub(super) prepared_query_execute: RuntimeFunction<'ctxt>,
    pub(super) prepared_query_bind_scalar_param: RuntimeFunction<'ctxt>,
    pub(super) prepared_query_bind_row_param: RuntimeFunction<'ctxt>,
    pub(super) prepared_query_add_ref: RuntimeFunction<'ctxt>,
    pub(super) prepared_query_remove_ref: RuntimeFunction<'ctxt>,
    
    pub(super) select_query_plan_new: RuntimeFunction<'ctxt>,
    pub(super) select_query_plan_set_where: RuntimeFunction<'ctxt>,
    pub(super) select_query_plan_prepare: RuntimeFunction<'ctxt>,
    pub(super) insert_query_plan_new: RuntimeFunction<'ctxt>,
    pub(super) insert_query_plan_prepare: RuntimeFunction<'ctxt>,
    pub(super) delete_query_plan_new: RuntimeFunction<'ctxt>,
    pub(super) delete_query_plan_set_where: RuntimeFunction<'ctxt>,
    pub(super) delete_query_plan_prepare: RuntimeFunction<'ctxt>,
    pub(super) update_query_plan_new: RuntimeFunction<'ctxt>,
    pub(super) update_query_plan_add_assignment: RuntimeFunction<'ctxt>,
    pub(super) update_query_plan_set_where: RuntimeFunction<'ctxt>,
    pub(super) update_query_plan_prepare: RuntimeFunction<'ctxt>,

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
        let long_type = context.i64_type();
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

        let new_array = Self::add_runtime_function(
            module,
            "__ql__QLArray_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
        );

        let add_array_ref = Self::add_runtime_function(
            module,
            "__ql__QLArray_add_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let remove_array_ref = Self::add_runtime_function(
            module,
            "__ql__QLArray_remove_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let index_array = Self::add_runtime_function(
            module,
            "__ql__QLArray_index",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into()], false),
        );

        let append_array = Self::add_runtime_function(
            module,
            "__ql__QLArray_append",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let pop_array = Self::add_runtime_function(
            module,
            "__ql__QLArray_pop",
            ptr_type.fn_type(&[ptr_type.into()], false),
        );

        let array_length = Self::add_runtime_function(
            module,
            "__ql__QLArray_length",
            int_type.fn_type(&[ptr_type.into()], false),
        );

        let init_dbs = Self::add_runtime_function(
            module,
            "__ql__init_dbs_from_args",
            void_type.fn_type(&[
                int_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let close_dbs = Self::add_runtime_function(
            module,
            "__ql__close_dbs",
            void_type.fn_type(&[
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let prepared_query_execute = Self::add_runtime_function(
            module,
            "__ql__PreparedQuery_execute",
            ptr_type.fn_type(&[ptr_type.into()], false),
        );

        let prepared_query_bind_scalar_param = Self::add_runtime_function(
            module,
            "__ql__PreparedQuery_bind_scalar_param",
            void_type.fn_type(&[
                ptr_type.into(),
                int_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let prepared_query_bind_row_param = Self::add_runtime_function(
            module,
            "__ql__PreparedQuery_bind_row_param",
            void_type.fn_type(&[
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
                ptr_type.into(),
            ], false),
        );

        let prepared_query_add_ref = Self::add_runtime_function(
            module,
            "__ql__PreparedQuery_add_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let prepared_query_remove_ref = Self::add_runtime_function(
            module,
            "__ql__PreparedQuery_remove_ref",
            void_type.fn_type(&[ptr_type.into()], false),
        );

        let select_query_plan_new = Self::add_runtime_function(
            module,
            "__ql__SelectQueryPlan_new",
            ptr_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
            ], false),
        );

        let select_query_plan_set_where = Self::add_runtime_function(
            module,
            "__ql__SelectQueryPlan_set_where",
            void_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let select_query_plan_prepare = Self::add_runtime_function(
            module,
            "__ql__SelectQueryPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let insert_query_plan_new = Self::add_runtime_function(
            module,
            "__ql__InsertQueryPlan_new",
            ptr_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
                bool_type.into(),
                ptr_type.into(),
            ], false),
        );

        let insert_query_plan_prepare = Self::add_runtime_function(
            module,
            "__ql__InsertQueryPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let delete_query_plan_new = Self::add_runtime_function(
            module,
            "__ql__DeleteQueryPlan_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into()], false),
        );

        let delete_query_plan_set_where = Self::add_runtime_function(
            module,
            "__ql__DeleteQueryPlan_set_where",
            void_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let delete_query_plan_prepare = Self::add_runtime_function(
            module,
            "__ql__DeleteQueryPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let update_query_plan_new = Self::add_runtime_function(
            module,
            "__ql__UpdateQueryPlan_new",
            ptr_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
            ], false),
        );

        let update_query_plan_add_assignment = Self::add_runtime_function(
            module,
            "__ql__UpdateQueryPlan_add_assignment",
            void_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let update_query_plan_set_where = Self::add_runtime_function(
            module,
            "__ql__UpdateQueryPlan_set_where",
            void_type.fn_type(&[
                ptr_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
        );

        let update_query_plan_prepare = Self::add_runtime_function(
            module,
            "__ql__UpdateQueryPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
        );

        let type_info_type = context.opaque_struct_type("QLTypeInfo");
        type_info_type.set_body(
            &[
                long_type.into(),      // size
                ptr_type.into(),       // elem_drop
                int_type.into(),       // num_columns
                ptr_type.into(),       // set_nth
                ptr_type.into(),       // get_nth
            ],
            false,
        );

        let int_type_info = module.add_global(
            type_info_type,
            Some(AddressSpace::default()),
            "__ql__int_type_info"
        );
        int_type_info.set_linkage(Linkage::External);

        let bool_type_info = module.add_global(
            type_info_type,
            Some(AddressSpace::default()),
            "__ql__bool_type_info"
        );
        bool_type_info.set_linkage(Linkage::External);

        let string_type_info = module.add_global(
            type_info_type,
            Some(AddressSpace::default()),
            "__ql__QLString_type_info"
        );
        string_type_info.set_linkage(Linkage::External);

        let array_type_info = module.add_global(
            type_info_type,
            Some(AddressSpace::default()),
            "__ql__QLArray_type_info"
        );
        array_type_info.set_linkage(Linkage::External);

        RuntimeFunctions {
            type_info_type,
            int_type_info,
            bool_type_info,
            string_type_info,
            array_type_info,

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
            new_array,
            add_array_ref,
            remove_array_ref,
            index_array,
            append_array,
            pop_array,
            array_length,

            init_dbs,
            close_dbs,
            
            prepared_query_execute,
            prepared_query_bind_scalar_param,
            prepared_query_bind_row_param,
            prepared_query_add_ref,
            prepared_query_remove_ref,
            
            select_query_plan_new,
            select_query_plan_set_where,
            select_query_plan_prepare,
            insert_query_plan_new,
            insert_query_plan_prepare,
            delete_query_plan_new,
            delete_query_plan_set_where,
            delete_query_plan_prepare,
            update_query_plan_new,
            update_query_plan_add_assignment,
            update_query_plan_set_where,
            update_query_plan_prepare,

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