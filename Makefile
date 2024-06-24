prog := rust-server

debug ?= 0

$(info debug is $(debug))

ifneq ($(debug), 0)
  release :=
  target :=debug
  extension :=debug
  rust_log :=debug
else
  release :=--release
  target :=release
  extension :=
  rust_log :=info
endif

build:
	cargo build $(release)

dev:
	RUST_LOG=$(rust_log) cargo watch -x "run -- $(prog) $(ARGS)"

test:
	cargo test -- --test-threads 1

protos:
	buf generate

migration:
	sqlx migrate run

all: protos test build

help:
	@echo "usage: make $(prog) [debug=1]"
