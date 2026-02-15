#include "qlstring.h"
#include "array.h"
#include "callable.h"
#include "memory.h"

void __ql__drop_value(void* value_ptr, QLTypeInfo* type_info) {
    switch (type_info->type) {
        case TYPE_STRING:
            __ql__QLString_remove_ref(*(QLString**)value_ptr);
            break;
        case TYPE_ARRAY:
            __ql__QLArray_remove_ref(*(QLArray**)value_ptr);
            break;
        case TYPE_STRUCT: {
            unsigned int num_fields = type_info->num_fields;
            for (unsigned int i = 0; i < num_fields; i++) {
                StructField field = type_info->fields[i];
                void* field_ptr = (char*)value_ptr + field.offset;
                __ql__drop_value(field_ptr, field.type_info);
            }
            break;
        }
        case TYPE_CALLABLE: {
            QLCallable* callable = *(QLCallable**)value_ptr;
            __ql__QLCallable_remove_ref(callable);
            break;
        }
        default:
            // Primitive types don't require special handling
            break;
    }
}
