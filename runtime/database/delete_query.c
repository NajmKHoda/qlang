#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "definitions.h"
#include "delete_query.h"

DeletePlan* __ql__DeletePlan_new(char* table_name) {
    DeletePlan* plan = malloc(sizeof(DeletePlan));
    plan->table_name = table_name;
    plan->has_where_clause = false;
    return plan;
}

void __ql__DeletePlan_set_where(DeletePlan* plan, char* column_name) {
    plan->has_where_clause = true;
    plan->where_column = column_name;
}

PreparedDelete* __ql__DeletePlan_prepare(sqlite3* db, DeletePlan* plan) {
    PreparedDelete* prepared_delete = malloc(sizeof(PreparedDelete));
    
    char sql[MAX_SQL_LENGTH];
    if (plan->has_where_clause) {
        sprintf(sql, "DELETE FROM %s WHERE %s = ?;", plan->table_name, plan->where_column);
    } else {
        sprintf(sql, "DELETE FROM %s;", plan->table_name);
    }

    sqlite3_prepare_v2(db, sql, -1, &prepared_delete->stmt, NULL);
    free(plan);
    return prepared_delete;
}


void __ql__PreparedDelete_bind_where(PreparedDelete* prepared_delete, QLType value_type, void* value) {
    __ql__bind_value(prepared_delete->stmt, 1, value_type, value);
}

void __ql__PreparedDelete_exec(PreparedDelete* prepared_delete) {
    sqlite3_step(prepared_delete->stmt);
    sqlite3_reset(prepared_delete->stmt);
}

void __ql__PreparedDelete_finalize(PreparedDelete* prepared_delete) {
    sqlite3_finalize(prepared_delete->stmt);
    free(prepared_delete);
}
