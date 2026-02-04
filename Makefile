.PHONY: build run install test clean help

build:
	cargo build --release

run:
	cargo run --release

install:
	cargo install --path .

test:
	cargo test

clean:
	cargo clean

help:
	@echo "mtop - Money Top 💰"
	@echo ""
	@echo "Available commands:"
	@echo "  make build    - Build release binary"
	@echo "  make run      - Run mtop (release)"
	@echo "  make install  - Install mtop locally"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make help     - Show this help message"
