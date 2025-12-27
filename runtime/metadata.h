#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

struct QLTypeInfo {
    unsigned long size;
    void (*elem_drop)(void* elem_ptr);
    void (*set_nth)(void* struct_ptr, unsigned int index, void* value_ptr);
};

#endif