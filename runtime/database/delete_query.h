#ifndef RUNTIME_DELETE_QUERY_H
#define RUNTIME_DELETE_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

// Represents a plan for deleting data from a table.
typedef struct DeletePlan {
    char* table_name;
    bool has_where_clause;
    char* where_column;
} DeletePlan;

// Represents a prepared DELETE statement.
typedef struct PreparedDelete {
    sqlite3_stmt* stmt;
} PreparedDelete;

// Creates a new DeletePlan for the specified table.
// @param table_name The name of the table to delete from.
DeletePlan* __ql__DeletePlan_new(char* table_name);

// Sets the WHERE clause for the delete plan.
// @param plan The delete plan to modify.
// @param column_name The name of the column to use in the WHERE clause
void __ql__DeletePlan_set_where(DeletePlan* plan, char* column_name);

// Prepares a delete statement from the given plan.
// @param plan The delete plan to prepare.
PreparedDelete* __ql__DeletePlan_prepare(DeletePlan* plan);


// Binds a value to the WHERE clause of the prepared delete statement.
// @param prepared_delete The prepared delete statement.
// @param value_type The type of the value to bind.
// @param value A pointer to the value to bind.
bool __ql__PreparedDelete_bind_where(
    PreparedDelete* prepared_delete,
    ColumnType value_type,
    void* value
);

// Executes the prepared delete statement.
// @param prepared_delete The prepared delete statement to execute.
bool __ql__PreparedDelete_exec(PreparedDelete* prepared_delete);

// Finalizes the prepared delete statement and frees associated resources.
// @param prepared_delete The prepared delete statement to finalize.
void __ql__PreparedDelete_finalize(PreparedDelete* prepared_delete);

#endif
