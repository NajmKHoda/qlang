#ifndef RUNTIME_UPDATE_QUERY_H
#define RUNTIME_UPDATE_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

// Represents a plan for updating data in a table.
typedef struct {
    char* table_name;
    unsigned int num_assignments;
    char** assign_columns;
    bool has_where_clause;
    char* where_column;
} UpdatePlan;

// Represents a prepared UPDATE statement.
typedef struct {
    sqlite3_stmt* stmt;
} PreparedUpdate;

// Creates a new UpdatePlan for the specified table and assignments.
// @param table_name The name of the table to update.
// @param num_assignments The number of column assignments in the update.
// @param assign_columns An array of column names being assigned in the update.
UpdatePlan* __ql__UpdatePlan_new(
    char* table_name,
    unsigned int num_assignments,
    char** assign_columns
);

// Sets the WHERE clause for the update plan.
// @param plan The update plan to modify.
// @param column_name The name of the column to use in the WHERE clause.
void __ql__UpdatePlan_set_where(UpdatePlan* plan, char* column_name);

// Prepares an UPDATE statement from the given plan.
// @param plan The update plan to prepare.
PreparedUpdate* __ql__UpdatePlan_prepare(UpdatePlan* plan);


// Binds a value to the WHERE clause of the prepared update statement.
// @param prepared_update The prepared update statement.
// @param value_type The type of the value to bind.
// @param value A pointer to the value to bind.
void __ql__PreparedUpdate_bind_where(
    PreparedUpdate* prepared_update,
    ColumnType value_type,
    void* value
);

// Binds a value to an assignment parameter of the prepared update statement.
// @param prepared_update The prepared update statement.
// @param index The index of the assignment to bind (0-based).
// @param value_type The type of the value to bind.
// @param value A pointer to the value to bind.
void __ql__PreparedUpdate_bind_assignment(
    PreparedUpdate* prepared_update,
    unsigned int index,
    ColumnType value_type,
    void* value
);

// Executes the prepared update statement.
// @param prepared_update The prepared update statement to execute.
void __ql__PreparedUpdate_exec(PreparedUpdate* prepared_update);

// Finalizes the prepared update statement and frees associated resources.
// @param prepared_update The prepared update statement to finalize.
void __ql__PreparedUpdate_finalize(PreparedUpdate* prepared_update);

#endif
