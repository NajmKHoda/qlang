#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

typedef struct SelectPlan {
    QLTypeInfo* struct_type_info;
    char* table_name;
    bool has_where_clause;
    char* where_column;
} SelectPlan;

typedef struct SelectIteratorState {
    char* sql; // Only set in master statements
    sqlite3_stmt* stmt;
    void* row_ptr;
    enum {
        SELECT_ITERATOR_NEXT,
        SELECT_ITERATOR_READY,
        SELECT_ITERATOR_EXHAUSTED
    } state;
} SelectIteratorState;

SelectPlan* __ql__SelectPlan_new(char* table_name, QLTypeInfo* struct_type_info);
void __ql__SelectPlan_set_where(SelectPlan* plan, char* column_name);
QLIterator* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan);
QLIterator* __ql__SelectIterator_activate(QLIterator* select_iterator);

void __ql__SelectIterator_bind_where(QLIterator* select_iterator, ColumnType value_type, void* value);
void __ql__SelectIterator_finalize(QLIterator* select_iterator);

#endif