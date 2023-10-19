#!make
SHELL:=/bin/bash

# pp - pretty print function
yellow := $(shell tput setaf 3)
normal := $(shell tput sgr0)
define pp
	@printf '$(yellow)$(1)$(normal)\n'
endef


help: Makefile
	@echo " Choose a command to run:"
	@sed -n 's/^##//p' $< | column -t -s ':' | sed -e 's/^/ /'


# DEV #############################################################################################

## withenv: 😭 CALL TARGETS LIKE THIS `make withenv RECIPE=dev.init`
withenv:
# NB: IT APPEARS THAT LOADING ENVIRONMENT VARIABLES INTO make SUUUUCKS.
# NB: THIS RECIPE IS A HACK TO MAKE IT WORK.
# NB: THAT'S WHY THIS MAKEFILE NEEDS TO BE CALLED LIKE `make withenv RECIPE=dev.init`
	test -e .env || cp .env.example .env
	bash -c 'set -o allexport; source .env; set +o allexport; make "$$RECIPE"'

## dev.init: 🌏 Initialize local dev environment
# If rdkafka compilation fails with SSL error then install openssl@1.1 or later and export:
# export LDFLAGS=-L/opt/homebrew/opt/openssl@1.1/lib
# export CPPFLAGS=-I/opt/homebrew/opt/openssl@1.1/include
dev.init: install
	$(call pp,install git hooks...)
	cargo install cargo-watch
	cargo test

## dev.kafka_init: 🥁 Init kafka topic
# dev.kafka_init:
# 	$(call pp,creating kafka topic...)
# 	cargo run --example kafka_create_topic

dev.chronos_ex:
	$(call pp,creating kafka topic...)
	cargo run --example chronos_ex

## pg.create: 🥁 Create database
pg.create:
	$(call pp,creating database...)
	cargo run --example pg_create_database

## pg.migrate: 🥁 Run migrations on database
pg.migrate:
	$(call pp,running migrations on database...)
	cargo run --package pg_mig --bin chronos-pg-migrations   

# TEST / DEPLOY ###################################################################################

## install: 🧹 Installs dependencies
install:
	$(call pp,pull rust dependencies...)
	rustup install "${RUST_VERSION}"
	rustup component add rust-src clippy llvm-tools-preview
	rustup toolchain install nightly
	rustup override set "${RUST_VERSION}"
	cargo install cargo2junit grcov
	cargo fetch

## build: 🧪 Compiles rust
build:
	$(call pp,build rust...)
	cargo build


## dev.run: 🧪 Runs rust app in watch mode
dev.run:
	$(call pp,run app...)
	cargo  watch -q -c -x 'run --package chronos_bin --bin chronos'

## run: 🧪 Runs rust app
run:
	$(call pp,run app...)
	cargo run --package chronos_bin --bin chronos

## run: 🧪 Runs rust app in release mode
run.release:
	$(call pp,run app...)
	cargo run --package chronos_bin --r --bin chronos 


## lint: 🧹 Checks for lint failures on rust
lint:
	$(call pp,lint rust...)
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets

## test.unit: 🧪 Runs unit tests
test.unit:
	$(call pp,rust unit tests...)
	cargo test

## test.unit.coverage: 🧪 Runs rust unit tests with coverage 'cobertura' and 'junit' reports
test.unit.coverage:
	$(call pp,rust unit tests...)
	sh scripts/coverage-report.sh

## docker.up: 🧪 Runs rust app in docker container along with kafka and postgres
docker.up:
	$(call pp,run app...)
	docker-compose --env-file /dev/null up -d

## docker.down: bring down the docker containers
docker.down:
	$(call pp,run app...)
	docker-compose down
# PHONY ###########################################################################################

# To force rebuild of not-file-related targets, make the targets "phony".
# A phony target is one that is not really the name of a file;
# Rather it is just a name for a recipe to be executed when you make an explicit request.
.PHONY: build
