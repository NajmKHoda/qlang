#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

typedef enum {
    TYPE_INT,
    TYPE_BOOL,
    TYPE_STRING,
    TYPE_ARRAY,
} QLType;

typedef struct {
    QLType type;
    unsigned int offset;
} StructField;

typedef struct {
    unsigned long size;
    void (*elem_drop)(void* elem_ptr);
    unsigned int num_fields;
    StructField* fields;
} QLTypeInfo;

#endif