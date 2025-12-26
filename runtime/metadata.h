#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

struct QLTypeInfo {
    unsigned long size;
    void (*elem_drop)(void* elem_ptr);
};

#endif