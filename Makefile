DESTDIR =
PREFIX  = /usr/local

all: target/release/uvm
build: target/release/uvm

target/release/uvm:
	cargo build --release --all

install: install-uvm

install-uvm: target/release/uvm
	mkdir -p "$(DESTDIR)$(PREFIX)/bin/"
	install -m755 -- target/release/uvm "$(DESTDIR)$(PREFIX)/bin/"

test: target/release/uvm
	cargo test --release $(CARGO_OPTS)

check: test

uninstall:
	-rm -f -- "$(DESTDIR)$(PREFIX)/bin/uvm-*"
	-rm -f -- "$(DESTDIR)$(PREFIX)/bin/uvm"

clean:
	cargo clean

help:
	@echo 'Available make targets:'
	@echo '  all         - build uvm (default)'
	@echo '  build       - build uvm'
	@echo '  clean       - run `cargo clean`'
	@echo '  install     - build and install uvm'
	@echo '  install-grlm - build and install uvm'
	@echo '  test        - run `cargo test`'
	@echo '  uninstall   - uninstall uvm'
	@echo '  help        - print this help'
	@echo
	@echo
	@echo 'Variables:'
	@echo '  DESTDIR  - A path that'\''s prepended to installation paths (default: "")'
	@echo '  PREFIX   - The installation prefix for everything except zsh completions (default: /usr/local)'
	@echo '  FEATURES - The cargo feature flags to use. Set to an empty string to disable git support'

.PHONY: all build target/release/uvm install-uvm \
	clean uninstall help
