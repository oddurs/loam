# loam — the targets CI runs, and the ones cargo does not cover.
#
# cairn keeps these in a Makefile and harrow in scripts/task. This follows
# cairn: loam's CI needs cairn and Python as well as cargo, which is what make
# is for, and `make check` is the name an agent will guess. Either way the rule
# is harrow's: CI runs exactly `make check`, so CI and a local run cannot drift.
CARGO   ?= cargo
PYTHON  ?= python3
CAIRN   ?= cairn
LOAM    := target/debug/loam

.PHONY: all build check test lint fmt audit conformance backlog docs install clean

all: build

build:
	$(CARGO) build --release

# Everything CI runs.
check: fmt lint test audit conformance backlog docs
	@echo
	@echo "check: green"

fmt:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets -- -D warnings

test:
	$(CARGO) test

# Advisories, licences and sources. Needs `cargo install cargo-deny`.
audit:
	$(CARGO) deny check --hide-inclusion-graph

# Two readers of the format over the corpus: the one in spec/, written from the
# specification, and loam itself. Needs `pip install pyyaml`.
conformance: $(LOAM)
	$(PYTHON) spec/conformance.py
	$(PYTHON) spec/conformance.py -- $(LOAM) reading
	$(PYTHON) spec/shared.py

# This backlog, and the roadmap rendered from it.
backlog:
	$(CAIRN) check --render --strict

# loam's own docs, checked by loam.
docs: $(LOAM)
	$(LOAM) check --strict --render
	$(LOAM) check --stale --quiet

$(LOAM): FORCE
	$(CARGO) build -q
FORCE:

install:
	$(CARGO) install --locked --path .

clean:
	$(CARGO) clean
