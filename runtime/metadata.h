#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

typedef enum {
    TYPE_INT,
    TYPE_BOOL,
    TYPE_STRING,
    TYPE_ARRAY,
    TYPE_STRUCT,
    TYPE_CALLABLE
} QLType;

typedef struct QLTypeInfo QLTypeInfo;

typedef struct StructField {
    unsigned int offset;
    QLTypeInfo* type_info;
} StructField;

typedef struct QLTypeInfo {
    QLType type;
    unsigned long size;
    unsigned int num_fields;
    StructField* fields;
} QLTypeInfo;

#endif