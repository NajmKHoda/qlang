# QLang

**QLang is a compiled, statically typed, and database-centric programming language** that combines general-purpose control flow with first-class SQL-like query expressions.

This repository contains:
- `compiler/`: parser, semantic analysis, and code generation.
- `runtime/`: C runtime used by generated programs.
- `main.ql`: sample entrypoint program.

## High-Level Overview

QLang is designed to make procedural database-querying code easier and safer for the modern programmer. Long-standing approaches typically involve many tedious steps:

- Constructing raw SQL strings
- Passing SQL strings to a library for runtime compilation
- Manually managing lifetimes of prepared statements
- Checking for query failures
- Accessing result columns that might be the wrong type or even nonexistent

In a world where databases are a critical building block for almost every piece of software out there, it's surprising to see that many languages and frameworks lack first-class support for database operations. But not QLang.

QLang's first-class support for queries allows for tight integration with its rigorous typing system. Queries are semantically checked against table definitions **at compile time** to avoid nasty runtime errors associated with other database frameworks. `SELECT` results carry typing information with them to ensure that all field accesses are well-defined and legal.

Additionally, QLang provides native support for **automatic error handling** and **transactions**, must-haves for any serious developer looking to tie up loose ends in their database operations. Transactions are abstracted as control structures similar to `try/catch` in other languages.

Here are some other features provided by QLang:
- Familar control flow (`if`, `for`, `while`, `break`, etc.).
- Procedural closures with `lambda() { ... }`.
- Prepared statements with `query() { ... }`.
- Builtin functions and methods for I/O, arrays, and iterators.

## Getting Started

Let `QLANG_PROJECT` be the path to the QLang project root (i.e. the directory this `README` is in).

1. Run `rustc --version` and `cargo --version` to check that the Rust language and Cargo package manager exist on your system. If not, [download `rustup`](https://rustup.rs/) to install them both.

2. Run `sqlite3 --version` to check if SQLite is installed on your system. If not, [head over to its download page](https://sqlite.org/download.html) to install the library on your system.

3. Run the following commands to build the QLang compiler and runtime:
```sh
cd QLANG_PROJECT
make
```

3. In your shell config file (`~/.zshrc` or `~/.bashrc`), append the following line (remember to swap `QLANG_PROJECT`!). Run `source` on the config file to refresh any running terminal sessions.
```sh
export PATH="QLANG_PROJECT/compiler/target/release/:$PATH"
```

4. The compiler is now ready. `cd` to a directory of your choosing and write a QLang program file. We'll call this program `main.ql`, though you're welcome to name it anything you want. The file extension does not have to be `.ql` either.

5. To compile your program, run the `qlang` command. Provide the filepath to your program and the desired filepath of the binary:
```sh
qlang main.ql main
```

6. To run your program, you'll need to set up a SQLite database file for each datasource. Use the `sqlite3` CLI to create each database file and `CREATE TABLE` statements to initialize tables that align with the program definitions. For this example, we'll assume a single datasource and database filepath `test.db`.

7. Run your program binary, providing it filepaths to each datasource in order. The first datasource is assigned the first filepath, the second datasource is assigned the second filepath, and so on:
```sh
./main test.db
```

Huzzah! You can now compile and run QLang programs on your system.

## Documentation

Come check out QLang's detailed [documentation](docs/README.md)!

[**QLang Statements**](docs/statements/README.md)

[**QLang Expressions**](docs/expressions/README.md)

[**QLang Builtin Functions and Methods**](docs/builtins/README.md)

[**QLang Queries**](docs/queries/README.md)

[**QLang Types**](docs/types/README.md)

[**Miscellaneous**](docs/misc/README.md)
