#include <stdbool.h>
#include <sqlite3.h>
#include <stdio.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "definitions.h"

void __ql__bind_value(sqlite3_stmt* stmt, unsigned int index, ColumnType value_type, void* value) {
    switch (value_type) {
        case COLUMN_STRING: {
            QLString* str = *(QLString**)value;
            fprintf(stderr, "bind string \"%.*s\" at index %d\n", str->length, str->raw_string, index);
            sqlite3_bind_text(stmt, index, str->raw_string, str->length, SQLITE_TRANSIENT);
            break;
        }
        case COLUMN_INT: {
            sqlite3_bind_int(stmt, index, *(int*)value);
            break;
        }
        case COLUMN_BOOL: {
            int as_int = *(bool*)value ? 1 : 0;
            sqlite3_bind_int(stmt, index, as_int);
            break;
        }
        default:
            break;
    }
}