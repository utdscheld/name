.DEFAULT_GOAL := run
PROJECT_DIR := .
BINARY_NAME := name
BUILD_DIR := $(PROJECT_DIR)/target/debug

# Cargo command
CARGO := cargo

CONFIG := configs/default.toml
GNUCFG := configs/mips-linux-gnu-as.toml
INPUT := .artifacts/mips_test.asm
OUTPUT := output.o

fmt:
	$(CARGO) fmt

build: fmt
	$(CARGO) build

run: build
	$(CARGO) run -- -l $(CONFIG) $(INPUT) $(OUTPUT)

run-gnu: build
	$(CARGO) run -- $(GNUCFG) $(INPUT) $(OUTPUT)

test: run
	$(CARGO) run -- $(GNUCFG) $(INPUT) gnu-$(OUTPUT)
	md5sum $(OUTPUT) gnu-$(OUTPUT)

clean:
	$(CARGO) clean
	rm -f $(BUILD_DIR)/$(BINARY_NAME)
	git clean -Xf



# Set the default rule to build and run the program
.PHONY: default
default: build run

# Help target to display available make targets
help:
	@echo "Available targets:"
	@echo "  make          - Build and Run"
	@echo "  make build    - Build NAME using Cargo"
	@echo "  make run      - Run NAME"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make help     - Display this help message"

# Ensure that 'help' is not treated as a file target
.PHONY: help
