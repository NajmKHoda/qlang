#ifndef RUNTIME_INSERT_QUERY_H
#define RUNTIME_INSERT_QUERY_H

#include <stdbool.h>
#include "database.h"

typedef struct QLTypeInfo QLTypeInfo;

// Represents a plan for inserting data into a table.
typedef struct InsertPlan {
    QLTypeInfo* struct_type_info;
    char* table_name;
} InsertPlan;

// Represents a prepared INSERT statement.
typedef struct PreparedInsert {
    sqlite3_stmt* stmt;
    QLTypeInfo* struct_type_info;
} PreparedInsert;

// Creates a new InsertPlan for the specified table and struct type information.
// @param table_name The name of the table to insert into.
// @param struct_type_info The type information of the struct being inserted.
InsertPlan* __ql__InsertPlan_new(char* table_name, QLTypeInfo* struct_type_info);

// Prepares an insert statement from the given plan.
// @param plan The insert plan to prepare.
PreparedInsert* __ql__InsertPlan_prepare(InsertPlan* plan);


// Executes the prepared insert statement for a single row of data.
// @param prepared_insert The prepared insert statement to execute.
// @param row A pointer to the struct representing the row to insert.
bool __ql__PreparedInsert_exec_row(PreparedInsert* prepared_insert, void* row);

// Executes the prepared insert statement for each element in the given array.
// @param prepared_insert The prepared insert statement to execute.
// @param array A pointer to the array of structs to insert.
bool __ql__PreparedInsert_exec_array(PreparedInsert* prepared_insert, QLArray* array);

// Finalizes the prepared insert statement and frees associated resources.
// @param prepared_insert The prepared insert statement to finalize.
void __ql__PreparedInsert_finalize(PreparedInsert* prepared_insert);

#endif
