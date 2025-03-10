.PHONY: build lint test docker-build docker-push

GIT_TAG := $(shell git rev-parse --short HEAD)

lint:
	cargo fmt
	cargo clippy

test: lint
	cargo test

build: test
	cargo build

docker-build: build
	docker build -t mbround18/gsm-reference:sha-$(GIT_TAG) .

docker-push: docker-build
	docker push mbround18/gsm-reference:sha-$(GIT_TAG)
