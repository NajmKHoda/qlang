#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

struct QLTypeInfo {
    unsigned long size;
    void (*elem_drop)(void* elem_ptr);

    // Only set for struct types
    unsigned int num_columns;
    void (*set_nth)(void* struct_ptr, unsigned int index, void* value_ptr);
    void (*get_nth)(void* struct_ptr, unsigned int index, int* datatype, void** out_value_ptr);
};

#endif