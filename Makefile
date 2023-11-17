.PHONY: list
SHELL := /bin/bash
export DOCKER_BUILDKIT=1

check:
	cargo check --workspace

list:
	@awk -F: '/^[A-z]/ {print $$1}' Makefile | sort

build:
	cargo build
	docker-compose build

push:
	docker-compose push

up:
	docker-compose -f docker-compose.yml up --detach --remove-orphans

down:
	docker-compose -f docker-compose.yml down --remove-orphans

run:
	cargo run -p scylladb-quick-demo-rs 20 80

apply:
	cd infra
	terraform apply

reset:
	docker-compose down --remove-orphans scylla
	docker-compose up --detach --force-recreate scylla
