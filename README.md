# QLang

QLang is a compiled, statically typed, database-centric programming language that combines general-purpose control flow with first-class SQL-like query expressions.

This repository contains:
- `compiler/`: parser, semantic analysis, and code generation.
- `runtime/`: C runtime used by generated programs.
- `main.ql`: sample entrypoint program.

## High-Level Overview

QLang provides:
- Program-level declarations for datasources, tables, structs, and functions.
- Typed statements and expressions with familiar control flow (`if`, `while`, `for`, `return`).
- Query expressions (`query { ... }`) for `select`, `insert`, `update`, and `delete`.
- Built-in functions and methods for I/O, arrays, and iterators.
- A type system with primitives, arrays, iterators, structs, and callable function types.
- Optional failure-aware callables using `failable`.

## Documentation

Detailed docs live in the [`docs/`](docs/README.md) folder:
- [`docs/statements/`](docs/statements/README.md)
- [`docs/expressions/`](docs/expressions/README.md)
- [`docs/builtins/`](docs/builtins/README.md)
- [`docs/queries/`](docs/queries/README.md)
- [`docs/types/`](docs/types/README.md)
- [`docs/misc/`](docs/misc/README.md)

## Status

This README is a starting skeleton. Each page in `docs/` is structured for gradual expansion with grammar snippets, semantic behavior, and runnable examples.
