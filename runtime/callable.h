#ifndef RUNTIME_CALLABLE_H
#define RUNTIME_CALLABLE_H

typedef enum CallableType {
    CALLABLE_PROCEDURAL,
    CALLABLE_SELECT,
    CALLABLE_INSERT,
    CALLABLE_UPDATE,
    CALLABLE_DELETE
} CallableType;

typedef struct QLCallable {
    void* invoke_fn;
    void* context_struct;
    struct QLTypeInfo* context_info;
    void* prepared_stmt;

    CallableType type;
    unsigned int ref_count;
} QLCallable;

extern const QLTypeInfo __ql__QLCallable_type_info;

QLCallable* __ql__QLCallable_new(void* invoke_fn, CallableType type, struct QLTypeInfo* captured_info);
void __ql__QLCallable_set_stmt(QLCallable* callable, void* prepared_stmt);
void __ql__QLCallable_capture(QLCallable* callable, unsigned int index, void* value_ptr);
void* __ql__QLCallable_get_fn(QLCallable* callable);
void* __ql__QLCallable_get_context(QLCallable* callable);
void* __ql__QLCallable_get_stmt(QLCallable* callable);
void __ql__QLCallable_add_ref(QLCallable* callable);
void __ql__QLCallable_remove_ref(QLCallable* callable);

#endif