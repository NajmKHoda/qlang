#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

typedef struct {
    unsigned int table_index;
    char* column_name;
} QColumn;

typedef struct {
    char* right_table_name;
    QColumn left_column;
    QColumn right_column;
} JoinClause;

typedef struct SelectPlan {
    QLTypeInfo* struct_type_info;
    char* table_name;
    unsigned int num_columns;
    QColumn* columns;
    unsigned int num_joins;
    JoinClause* join_clauses;
    bool has_where_clause;
    QColumn where_column;
    bool has_limit_clause;
    bool has_offset_clause;
} SelectPlan;

typedef struct SelectIteratorState {
    char* sql; // Only set in master statements
    sqlite3_stmt* stmt;
    void* row_ptr;
    unsigned int where_bind_index;
    unsigned int limit_bind_index;
    unsigned int offset_bind_index;
    enum {
        SELECT_ITERATOR_NEXT,
        SELECT_ITERATOR_READY,
        SELECT_ITERATOR_EXHAUSTED
    } state;
} SelectIteratorState;

SelectPlan* __ql__SelectPlan_new(char* table_name, unsigned int num_columns, unsigned int num_joins, QLTypeInfo* struct_type_info);
void __ql__SelectPlan_set_column(SelectPlan* plan, unsigned int index, unsigned int table_index, char* column_name);
void __ql__SelectPlan_set_join(
    SelectPlan* plan,
    unsigned int index,
    char* right_table_name,
    unsigned int left_table_index,
    char* left_column_name,
    unsigned int right_table_index,
    char* right_column_name
);
void __ql__SelectPlan_set_where(SelectPlan* plan, unsigned int table_index, char* column_name);
void __ql__SelectPlan_set_limit(SelectPlan* plan);
void __ql__SelectPlan_set_offset(SelectPlan* plan);
QLIterator* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan);
QLIterator* __ql__SelectIterator_activate(QLIterator* select_iterator);

void __ql__SelectIterator_bind_where(QLIterator* select_iterator, ColumnType value_type, void* value);
void __ql__SelectIterator_bind_limit(QLIterator* select_iterator, void* value);
void __ql__SelectIterator_bind_offset(QLIterator* select_iterator, void* value);
void __ql__SelectIterator_finalize(QLIterator* select_iterator);

#endif