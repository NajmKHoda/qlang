#ifndef RUNTIME_DB_DEFINITIONS
#define RUNTIME_DB_DEFINITIONS

#include "../metadata.h"

#define MAX_SQL_LENGTH 1024

void __ql__bind_value(sqlite3_stmt* stmt, unsigned int index, QLType value_type, void* value);

#endif