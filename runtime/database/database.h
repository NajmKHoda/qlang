#ifndef RUNTIME_DATABASE_H
#define RUNTIME_DATABASE_H

#include <stdbool.h>

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals);
void __ql__close_dbs(int num_dbs, sqlite3*** db_globals);

#endif
