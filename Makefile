.PHONY: build lint test

lint:
	cargo fmt
	cargo clippy

test: lint
	cargo test

build: test
	cargo build