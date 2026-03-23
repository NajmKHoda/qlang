#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct QLIterator QLIterator;

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

typedef struct {
    QLTypeInfo* struct_type_info;
    char* table_name;
    bool has_where_clause;
    char* where_column;
} SelectPlan;

typedef enum PreparedSelectState {
    PREPARED_SELECT_STATE_READY,
    PREPARED_SELECT_STATE_NEXT,
    PREPARED_SELECT_STATE_EXHAUSTED
} PreparedSelectState;

typedef struct {
    char* sql; // Only set in master statements
    sqlite3_stmt* stmt;
    QLTypeInfo* struct_type_info;
    void* row_ptr;
    PreparedSelectState state;
    unsigned int ref_count;
} PreparedSelect;

SelectPlan* __ql__SelectPlan_new(char* table_name, QLTypeInfo* struct_type_info);
void __ql__SelectPlan_set_where(SelectPlan* plan, char* column_name);
PreparedSelect* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan);
PreparedSelect* __ql__PreparedSelect_copy_if_needed(PreparedSelect* prepared_select);

void __ql__PreparedSelect_bind_where(PreparedSelect* prepared_select, ColumnType value_type, void* value);
QLIterator* __ql__PreparedSelect_execute(PreparedSelect* prepared_select);
void __ql__PreparedSelect_finalize(PreparedSelect* prepared_select);

#endif