CC=clang
DEBUG=true
RUNTIME_OBJ=./out/runtime.o
RUNTIME_SRC=./runtime/*.c
COMPILER_MANIFEST=./compiler/Cargo.toml
PROGRAM_SRC=./main.ql
PROGRAM_OBJ=./out/main

ifeq ($(DEBUG),true)
	COMPILER_OBJ=./compiler/target/debug/db-lang
else
	COMPILER_OBJ=./compiler/target/release/db-lang
endif

$(PROGRAM_OBJ): $(PROGRAM_SRC) compiler
	@$(COMPILER_OBJ) $(PROGRAM_SRC) $(PROGRAM_OBJ)

run: $(PROGRAM_OBJ)
	@$(PROGRAM_OBJ)

compiler: runtime $(COMPILER_OBJ)

runtime: $(RUNTIME_OBJ)

.PHONY: $(COMPILER_OBJ)
$(COMPILER_OBJ):
	cargo build --manifest-path=$(COMPILER_MANIFEST) $(if $(DEBUG),,--release)

$(RUNTIME_OBJ): $(RUNTIME_SRC)
	@mkdir -p out
	@$(CC) -r $(RUNTIME_SRC) -o $(RUNTIME_OBJ)

clean:
	@rm -rf out/
	@cargo clean --manifest-path=$(COMPILER_MANIFEST)
