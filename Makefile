.PHONY: help build test fmt clippy clean regenerate doc setup-examples run-example

help:
	@echo "Firecracker Rust SDK - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build           - Build the library"
	@echo "  make test            - Run tests"
	@echo "  make fmt             - Format code"
	@echo "  make clippy          - Run clippy linter"
	@echo "  make doc             - Generate and open documentation"
	@echo "  make setup-examples  - Download test images for examples"
	@echo "  make run-example     - Run the simple_vm_alpine example"
	@echo "  make regenerate      - Regenerate SDK from firecracker.yaml"
	@echo "  make clean           - Clean build artifacts"

build:
	cargo build

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

doc:
	cargo doc --open

setup-examples:
	chmod +x examples/fetch_test_images.sh
	./examples/fetch_test_images.sh

run-example: setup-examples
	cargo run --example simple_vm_alpine

regenerate:
	./regenerate.sh

clean:
	cargo clean
