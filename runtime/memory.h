#ifndef RUNTIME_MEMORY_H
#define RUNTIME_MEMORY_H

typedef struct QLTypeInfo QLTypeInfo;

void __ql__drop_value(void* value_ptr, QLTypeInfo* type_info);

#endif