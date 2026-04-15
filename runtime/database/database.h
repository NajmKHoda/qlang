#ifndef RUNTIME_DATABASE_H
#define RUNTIME_DATABASE_H

#include <stdbool.h>

#define MAX_SQL_LENGTH 512

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

// Global SQLite database connection.
// All databases are attached to this connection.
extern sqlite3* __ql__sqlite;

// Initializes one SQLite database per datasource from command line arguments.
// @param argc The number of command line arguments.
// @param argv The command line arguments.
// @param num_dbs The number of datasources in the QLang program.
void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals);

// Closes all opened databases.
void __ql__close_dbs();

// Binds a QLang value to the specified index in the SQLite statement.
// @param stmt The SQLite statement.
// @param index The index of the parameter to bind.
// @param value_type The type of the value to bind.
// @param value A pointer to the value to bind.
void __ql__bind_value(sqlite3_stmt* stmt, unsigned int index, ColumnType value_type, void* value);

// Opens a SAVEPOINT with an internal name derived from savepoint_id.
// Returns true on success and false on SQLite error.
bool __ql__db_savepoint(unsigned int savepoint_id);

// Releases a SAVEPOINT with an internal name derived from savepoint_id.
// Returns true on success and false on SQLite error.
bool __ql__db_release_savepoint(unsigned int savepoint_id);

// Rolls back to a SAVEPOINT with an internal name derived from savepoint_id.
// Returns true on success and false on SQLite error.
bool __ql__db_rollback_to_savepoint(unsigned int savepoint_id);

#endif
