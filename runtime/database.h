#ifndef RUNTIME_TABLE_H
#define RUNTIME_TABLE_H

typedef struct QLTypeInfo QLTypeInfo;
typedef struct QLArray QLArray;
typedef struct sqlite3 sqlite3;

typedef enum {
    QUERY_INTEGER,
    QUERY_STRING
} QueryDataType;

typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    struct {
        _Bool is_present;
        char* column_name;
        QueryDataType column_type;
        void* value;
    } where;
} QueryPlan;

sqlite3* __ql__init_database(char* db_filename);

QueryPlan* __ql__QueryPlan_new(char* table_name, QLTypeInfo* struct_type_info);
void __ql__QueryPlan_set_where(
    QueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
QLArray* __ql__QueryPlan_execute(sqlite3* db, QueryPlan* plan);

#endif