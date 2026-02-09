use inkwell::AddressSpace;
use inkwell::types::StructType;
use inkwell::{context::Context};
use inkwell::module::{Linkage, Module};
use inkwell::values::{FunctionValue, GlobalValue};

pub(super) struct RuntimeFunctions<'ctxt> {
    pub(super) type_info_type: StructType<'ctxt>,
    pub(super) struct_field_type: StructType<'ctxt>,
    pub(super) int_type_info: GlobalValue<'ctxt>,
    pub(super) bool_type_info: GlobalValue<'ctxt>,
    pub(super) string_type_info: GlobalValue<'ctxt>,
    pub(super) array_type_info: GlobalValue<'ctxt>,

    pub(super) print_integer: FunctionValue<'ctxt>,
    pub(super) print_boolean: FunctionValue<'ctxt>,
    pub(super) print_string: FunctionValue<'ctxt>,
    pub(super) input_integer: FunctionValue<'ctxt>,
    pub(super) input_string: FunctionValue<'ctxt>,

    pub(super) new_string: FunctionValue<'ctxt>,
    pub(super) add_string_ref: FunctionValue<'ctxt>,
    pub(super) remove_string_ref: FunctionValue<'ctxt>,
    pub(super) concat_string: FunctionValue<'ctxt>,
    pub(super) compare_string: FunctionValue<'ctxt>,

    pub(super) new_array: FunctionValue<'ctxt>,
    pub(super) add_array_ref: FunctionValue<'ctxt>,
    pub(super) remove_array_ref: FunctionValue<'ctxt>,
    pub(super) index_array: FunctionValue<'ctxt>,
    pub(super) append_array: FunctionValue<'ctxt>,
    pub(super) array_length: FunctionValue<'ctxt>,
    pub(super) pop_array: FunctionValue<'ctxt>,

    pub(super) init_dbs: FunctionValue<'ctxt>,
    pub(super) close_dbs: FunctionValue<'ctxt>,

    // Delete query functions
    pub(super) delete_plan_new: FunctionValue<'ctxt>,
    pub(super) delete_plan_set_where: FunctionValue<'ctxt>,
    pub(super) delete_plan_prepare: FunctionValue<'ctxt>,
    pub(super) prepared_delete_bind_where: FunctionValue<'ctxt>,
    pub(super) prepared_delete_exec: FunctionValue<'ctxt>,
    pub(super) prepared_delete_finalize: FunctionValue<'ctxt>,

    // Insert query functions
    pub(super) insert_plan_new: FunctionValue<'ctxt>,
    pub(super) insert_plan_prepare: FunctionValue<'ctxt>,
    pub(super) prepared_insert_exec_row: FunctionValue<'ctxt>,
    pub(super) prepared_insert_exec_array: FunctionValue<'ctxt>,
    pub(super) prepared_insert_finalize: FunctionValue<'ctxt>,

    // Select query functions
    pub(super) select_plan_new: FunctionValue<'ctxt>,
    pub(super) select_plan_set_where: FunctionValue<'ctxt>,
    pub(super) select_plan_prepare: FunctionValue<'ctxt>,
    pub(super) prepared_select_bind_where: FunctionValue<'ctxt>,
    pub(super) prepared_select_execute: FunctionValue<'ctxt>,
    pub(super) prepared_select_finalize: FunctionValue<'ctxt>,

    // Update query functions
    pub(super) update_plan_new: FunctionValue<'ctxt>,
    pub(super) update_plan_set_where: FunctionValue<'ctxt>,
    pub(super) update_plan_prepare: FunctionValue<'ctxt>,
    pub(super) prepared_update_bind_where: FunctionValue<'ctxt>,
    pub(super) prepared_update_bind_assignment: FunctionValue<'ctxt>,
    pub(super) prepared_update_exec: FunctionValue<'ctxt>,
    pub(super) prepared_update_finalize: FunctionValue<'ctxt>,
}

impl<'ctxt> RuntimeFunctions<'ctxt> {
    pub(super) fn new(context: &'ctxt Context, module: &Module<'ctxt>) -> Self {
        let void_type = context.void_type();
        let int_type = context.i32_type();
        let long_type = context.i64_type();
        let bool_type = context.bool_type();
        let ptr_type = context.ptr_type(Default::default());

        let print_integer = module.add_function(
            "printi",
            void_type.fn_type(&[int_type.into()], false),
            Some(Linkage::External),
        );

        let print_boolean = module.add_function(
            "printb",
            void_type.fn_type(&[bool_type.into()], false),
            Some(Linkage::External),
        );

        let print_string = module.add_function(
            "prints",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let input_integer = module.add_function(
            "inputi",
            int_type.fn_type(&[], false),
            Some(Linkage::External),
        );

        let input_string = module.add_function(
            "inputs",
            ptr_type.fn_type(&[], false),
            Some(Linkage::External),
        );

        let new_string = module.add_function(
            "__ql__QLString_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into(), bool_type.into()], false),
            Some(Linkage::External),
        );

        let add_string_ref = module.add_function(
            "__ql__QLString_add_ref",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let remove_string_ref = module.add_function(
            "__ql__QLString_remove_ref",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let concat_string = module.add_function(
            "__ql__QLString_concat",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let compare_string = module.add_function(
            "__ql__QLString_compare",
            int_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let new_array = module.add_function(
            "__ql__QLArray_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let add_array_ref = module.add_function(
            "__ql__QLArray_add_ref",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let remove_array_ref = module.add_function(
            "__ql__QLArray_remove_ref",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let index_array = module.add_function(
            "__ql__QLArray_index",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into()], false),
            Some(Linkage::External),
        );

        let append_array = module.add_function(
            "__ql__QLArray_append",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let pop_array = module.add_function(
            "__ql__QLArray_pop",
            ptr_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let array_length = module.add_function(
            "__ql__QLArray_length",
            int_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let init_dbs = module.add_function(
            "__ql__init_dbs_from_args",
            void_type.fn_type(&[
                int_type.into(),
                ptr_type.into(),
                int_type.into(),
                ptr_type.into(),
            ], false),
            Some(Linkage::External),
        );

        let close_dbs = module.add_function(
            "__ql__close_dbs",
            void_type.fn_type(&[int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        // Delete query functions
        let delete_plan_new = module.add_function(
            "__ql__DeletePlan_new",
            ptr_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let delete_plan_set_where = module.add_function(
            "__ql__DeletePlan_set_where",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let delete_plan_prepare = module.add_function(
            "__ql__DeletePlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_delete_bind_where = module.add_function(
            "__ql__PreparedDelete_bind_where",
            void_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_delete_exec = module.add_function(
            "__ql__PreparedDelete_exec",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_delete_finalize = module.add_function(
            "__ql__PreparedDelete_finalize",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        // Insert query functions
        let insert_plan_new = module.add_function(
            "__ql__InsertPlan_new",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let insert_plan_prepare = module.add_function(
            "__ql__InsertPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_insert_exec_row = module.add_function(
            "__ql__PreparedInsert_exec_row",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_insert_exec_array = module.add_function(
            "__ql__PreparedInsert_exec_array",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_insert_finalize = module.add_function(
            "__ql__PreparedInsert_finalize",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        // Select query functions
        let select_plan_new = module.add_function(
            "__ql__SelectPlan_new",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let select_plan_set_where = module.add_function(
            "__ql__SelectPlan_set_where",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let select_plan_prepare = module.add_function(
            "__ql__SelectPlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_select_bind_where = module.add_function(
            "__ql__PreparedSelect_bind_where",
            void_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_select_execute = module.add_function(
            "__ql__PreparedSelect_execute",
            ptr_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_select_finalize = module.add_function(
            "__ql__PreparedSelect_finalize",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        // Update query functions
        let update_plan_new = module.add_function(
            "__ql__UpdatePlan_new",
            ptr_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let update_plan_set_where = module.add_function(
            "__ql__UpdatePlan_set_where",
            void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let update_plan_prepare = module.add_function(
            "__ql__UpdatePlan_prepare",
            ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_update_bind_where = module.add_function(
            "__ql__PreparedUpdate_bind_where",
            void_type.fn_type(&[ptr_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_update_bind_assignment = module.add_function(
            "__ql__PreparedUpdate_bind_assignment",
            void_type.fn_type(&[ptr_type.into(), int_type.into(), int_type.into(), ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_update_exec = module.add_function(
            "__ql__PreparedUpdate_exec",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let prepared_update_finalize = module.add_function(
            "__ql__PreparedUpdate_finalize",
            void_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );

        let type_info_type = context.opaque_struct_type("QLTypeInfo");
        type_info_type.set_body(
            &[
                long_type.into(),      // size
                ptr_type.into(),       // elem_drop
                int_type.into(),       // num_fields
                ptr_type.into(),       // fields
            ],
            false,
        );

        let struct_field_type = context.opaque_struct_type("StructField");
        struct_field_type.set_body(
            &[
                int_type.into(),       // type enum
                int_type.into(),       // offset
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
            struct_field_type,
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

            delete_plan_new,
            delete_plan_set_where,
            delete_plan_prepare,
            prepared_delete_bind_where,
            prepared_delete_exec,
            prepared_delete_finalize,

            insert_plan_new,
            insert_plan_prepare,
            prepared_insert_exec_row,
            prepared_insert_exec_array,
            prepared_insert_finalize,

            select_plan_new,
            select_plan_set_where,
            select_plan_prepare,
            prepared_select_bind_where,
            prepared_select_execute,
            prepared_select_finalize,

            update_plan_new,
            update_plan_set_where,
            update_plan_prepare,
            prepared_update_bind_where,
            prepared_update_bind_assignment,
            prepared_update_exec,
            prepared_update_finalize,
        }
    }
}