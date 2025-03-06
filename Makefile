.PHONY: build lint test docker-build docker-push

lint:
	cargo fmt
	cargo clippy

test: lint
	cargo test

build: test
	cargo build


docker-build: build
	docker build -t mbround18/gsm-reference:latest .

docker-push: docker-build
	docker push mbround18/gsm-reference:latest