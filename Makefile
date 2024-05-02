prog := rust-server

debug ?= 0

$(info debug is $(debug))

ifdef debug
  release :=
  target :=debug
  extension :=debug
else
  release :=--release
  target :=release
  extension :=
endif

build:
	cargo build $(release)

dev:
	cargo watch -x "run -- $(prog) $(ARGS)"

test:
	cargo test -- --nocapture

protos:
	buf generate

all: protos test build

help:
	@echo "usage: make $(prog) [debug=1]"