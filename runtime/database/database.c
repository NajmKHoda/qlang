#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "database.h"

sqlite3* __ql__sqlite = NULL;

void __ql__init_dbs(int argc, char** argv, int num_dbs) {
    if (num_dbs == 0) {
        // No databases to open
        return;
    }

    argc--; argv++;
    if (argc < num_dbs) {
        fprintf(stderr, "Expected %d database file paths, got %d\n", num_dbs, argc);
        exit(1);
    }

    // Open the first database as the main database
    if (sqlite3_open(argv[0], &__ql__sqlite) != SQLITE_OK) {
        fprintf(stderr, "Cannot open database '%s':\n\t%s\n", argv[0], sqlite3_errmsg(__ql__sqlite));
        sqlite3_close(__ql__sqlite);
        exit(1);
    }

    // Attach the remaining databases
    for (int i = 1; i < argc; i++) {
        char attachSql[MAX_SQL_LENGTH];
        sprintf(attachSql, "ATTACH DATABASE '%s' AS db%u;", argv[i], i);
        if (sqlite3_exec(__ql__sqlite, attachSql, NULL, NULL, NULL) != SQLITE_OK) {
            fprintf(stderr, "Cannot attach database '%s':\n\t%s\n", argv[i], sqlite3_errmsg(__ql__sqlite));
            sqlite3_close(__ql__sqlite);
            exit(1);
        }
    }
}

void __ql__close_dbs() {
    sqlite3_close(__ql__sqlite);
    __ql__sqlite = NULL;
}

void __ql__bind_value(sqlite3_stmt* stmt, unsigned int index, ColumnType value_type, void* value) {
    switch (value_type) {
        case COLUMN_STRING: {
            QLString* str = *(QLString**)value;
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