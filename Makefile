.PHONY: list
SHELL := /bin/bash
export DOCKER_BUILDKIT=1

check:
	cargo check --workspace

list:
	@awk -F: '/^[A-z]/ {print $$1}' Makefile | sort

build:
	cargo build
	docker compose build

push:
	docker compose push

up:
	docker compose up --detach --force-recreate scylla demo

down:
	docker compose down --remove-orphans

run:
	cargo run -p scylladb-quick-demo-rs 80 20 30

kill:
	ps aux | grep "target/debug/scylladb-quick-demo-[rs]" | awk '{print $$2}' | xargs kill -9

reset:
	docker compose down --remove-orphans scylla
	docker compose up --detach --force-recreate scylla
