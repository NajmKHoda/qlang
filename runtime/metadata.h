#ifndef RUNTIME_METADATA_H
#define RUNTIME_METADATA_H

typedef struct QLTypeInfo QLTypeInfo;

typedef enum {
    COLUMN_INT,
    COLUMN_BOOL,
    COLUMN_STRING,
    COLUMN_REAL
} ColumnType;

typedef struct QLTypeInfo {
    unsigned long size;
    unsigned int num_fields;
    void (*copy)(void*);
    void (*drop)(void*);
    void* (*get_nth)(void*, unsigned int, ColumnType*);
    void (*set_nth)(void*, unsigned int, void*);
} QLTypeInfo;

#endif