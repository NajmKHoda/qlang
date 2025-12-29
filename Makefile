CC=clang
DEBUG=true
RUNTIME_OBJ=./out/runtime.o
RUNTIME_SRC=./runtime/*.c
RUNTIME_HEADERS=./runtime/*.h
COMPILER_MANIFEST=./compiler/Cargo.toml
PROGRAM_SRC=./main.ql
PROGRAM_OBJ=./out/main

ARGS ?=

ifeq ($(DEBUG),true)
	COMPILER_OBJ=./compiler/target/debug/db-lang
else
	COMPILER_OBJ=./compiler/target/release/db-lang
endif

.DEFAULT_GOAL := all
all: $(PROGRAM_OBJ)

$(PROGRAM_OBJ): $(PROGRAM_SRC) $(COMPILER_OBJ) $(RUNTIME_OBJ)
	@$(COMPILER_OBJ) $(PROGRAM_SRC) $(PROGRAM_OBJ)

$(COMPILER_OBJ): compiler

$(RUNTIME_OBJ): $(RUNTIME_SRC) $(RUNTIME_HEADERS)
	@mkdir -p out
	@$(CC) -r $(RUNTIME_SRC) -o $(RUNTIME_OBJ)

.PHONY: compiler run clean

compiler:
	cargo build --manifest-path=$(COMPILER_MANIFEST) $(if $(DEBUG),,--release)

run: $(PROGRAM_OBJ)
	@$(PROGRAM_OBJ) $(ARGS)

clean:
	@rm -rf out/
	@cargo clean --manifest-path=$(COMPILER_MANIFEST)
