.PHONY: build lint test docker-build docker-push

GIT_TAG := $(shell git rev-parse --short HEAD)
export COMPOSE_BAKE=true

lint:
	cargo fmt
	cargo clippy
	@if command -v npx > /dev/null 2>&1; then npx -y prettier --write .; fi


test: lint
	cargo test

build: test
	cargo build

docker-build: build
	docker build -t mbround18/gsm-reference:sha-$(GIT_TAG) .

docker-push: docker-build
	docker push mbround18/gsm-reference:sha-$(GIT_TAG)
