.DEFAULT_GOAL := run
PROJECT_DIR := .
BINARY_NAME := name
BUILD_DIR := $(PROJECT_DIR)/target/debug

# Cargo command
CARGO := cargo

build:
	$(CARGO) build

CONFIG := configs/default.toml
INPUT := .artifacts/mips_test.asm
OUTPUT := output.o

run:
	$(CARGO) run $(CONFIG) $(INPUT) $(OUTPUT)

fmt:
	$(CARGO) fmt

clean:
	$(CARGO) clean
	rm -f $(BUILD_DIR)/$(BINARY_NAME)

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
