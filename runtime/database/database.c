#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"

void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals) {
    argc--; argv++;
    if (argc < num_dbs) {
        fprintf(stderr, "Expected %d database file paths, got %d\n", num_dbs, argc);
        exit(1);
    }
    for (int i = 0; i < argc; i++) {
        sqlite3* db;
        if (sqlite3_open(argv[i], &db) != SQLITE_OK) {
            fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
            sqlite3_close(db);
            for (int j = 0; j < i; j++) {
                sqlite3_close(*(db_globals[j]));
            }
            exit(1);
        }
        *(db_globals[i]) = db;
    }
}

void __ql__close_dbs(int num_dbs, sqlite3*** db_globals) {
    for (int i = 0; i < num_dbs; i++) {
        sqlite3_close(*(db_globals[i]));
    }
}
