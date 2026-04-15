#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

// Represents a column in the SELECT query.
typedef struct QColumn {
    unsigned int table_index;
    char* column_name;
} QColumn;

// Represents a JOIN clause in the SELECT query.
typedef struct JoinClause {
    char* right_table_name;
    QColumn left_column;
    QColumn right_column;
} JoinClause;

// Represents a plan for executing a SELECT query.
typedef struct SelectPlan {
    QLTypeInfo* struct_type_info;   // Type info of the capturing struct.
    char* table_name;               // The name of the starting table.
    unsigned int num_columns;       // The number of columns being selected.
    QColumn* columns;               // The columns being selected for.

    unsigned int num_joins;         // The number of JOIN clauses in the query.
    JoinClause* join_clauses;       // The JOIN clauses in the query.

    bool has_where_clause;          // Indicates if the plan has a WHERE clause.
    QColumn where_column;           // The column used in the WHERE clause, if present.

    bool has_limit_clause;          // Indicates if the plan has a LIMIT clause.
    bool has_offset_clause;         // Indicates if the plan has an OFFSET clause.
} SelectPlan;

// QLIterator state for executing a SELECT query (SelectIterator).
typedef struct SelectIteratorState {
    char* sql;                          // Only set in master statements for cloning.
    sqlite3_stmt* stmt;                 // The prepared statement for this SELECT query.
    void* row_ptr;                      // Next row buffer.
    unsigned int where_bind_index;      // The index of the WHERE bind parameter (if present).
    unsigned int limit_bind_index;      // The index of the LIMIT bind parameter (if present).
    unsigned int offset_bind_index;     // The index of the OFFSET bind parameter (if present).
    enum {                              // The state of the iterator.
        SELECT_ITERATOR_NEXT,
        SELECT_ITERATOR_READY,
        SELECT_ITERATOR_EXHAUSTED
    } state;
} SelectIteratorState;

// Creates a new SelectPlan for the specified table.
// @param table_name The name of the table to select from.
// @param num_columns The number of columns being selected.
// @param num_joins The number of JOIN clauses in the query.
// @param struct_type_info The type information of the capturing struct.
SelectPlan* __ql__SelectPlan_new(char* table_name, unsigned int num_columns, unsigned int num_joins, QLTypeInfo* struct_type_info);

// Sets the column at the specified index in the select plan.
// @param plan The select plan to modify.
// @param index The index of the column to set (0-based).
// @param table_index The index of the table this column belongs to.
// @param column_name The name of the column to select.
void __ql__SelectPlan_set_column(SelectPlan* plan, unsigned int index, unsigned int table_index, char* column_name);

// Sets a JOIN clause at the specified index in the select plan.
// @param plan The select plan to modify.
// @param index The index of the JOIN clause to set (0-based).
// @param right_table_name The name of the right table in the JOIN.
// @param left_table_index The index of the left table in the JOIN.
// @param left_column_name The name of the column from the left table in the JOIN condition.
// @param right_table_index The index of the right table in the JOIN.
// @param right_column_name The name of the column from the right table in the JOIN condition
void __ql__SelectPlan_set_join(
    SelectPlan* plan,
    unsigned int index,
    char* right_table_name,
    unsigned int left_table_index,
    char* left_column_name,
    unsigned int right_table_index,
    char* right_column_name
);

// Sets the WHERE clause for the select plan.
// @param plan The select plan to modify.
// @param table_index The index of the table for the WHERE clause.
// @param column_name The name of the column for the WHERE clause.
void __ql__SelectPlan_set_where(SelectPlan* plan, unsigned int table_index, char* column_name);

// Sets the LIMIT clause for the select plan.
// @param plan The select plan to modify.
void __ql__SelectPlan_set_limit(SelectPlan* plan);

// Sets the OFFSET clause for the select plan.
// @param plan The select plan to modify.
void __ql__SelectPlan_set_offset(SelectPlan* plan);

// Prepares an (unactivated)SelectIterator from the given plan.
// @param plan The select plan to prepare.
QLIterator* __ql__SelectPlan_prepare(SelectPlan* plan);


// Activates the given SelectIterator for use.
// Resets the underlying prepared statement, or clones if in use.
// @param select_iterator The SelectIterator to activate.
QLIterator* __ql__SelectIterator_activate(QLIterator* select_iterator);

// Binds a value to the WHERE clause of the given SelectIterator.
// @param select_iterator The SelectIterator to bind the value for.
// @param value_type The type of the value to bind.
// @param value A pointer to the value to bind.
void __ql__SelectIterator_bind_where(QLIterator* select_iterator, ColumnType value_type, void* value);

// Binds a value to the LIMIT clause of the given SelectIterator.
// @param select_iterator The SelectIterator to bind the value for.
// @param value A pointer to the integer value to bind for the LIMIT clause.
void __ql__SelectIterator_bind_limit(QLIterator* select_iterator, void* value);

// Binds a value to the OFFSET clause of the given SelectIterator.
// @param select_iterator The SelectIterator to bind the value for.
// @param value A pointer to the integer value to bind for the OFFSET clause.
void __ql__SelectIterator_bind_offset(QLIterator* select_iterator, void* value);

// Finalizes the given SelectIterator and frees associated resources.
// @param select_iterator The SelectIterator to finalize.
void __ql__SelectIterator_finalize(QLIterator* select_iterator);

#endif