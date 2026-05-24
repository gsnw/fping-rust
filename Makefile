BINARY_NAME := fping
VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH := $(shell uname -m | sed 's/x86_64/x86_64/;s/aarch64/arm64/')
ARCHIVE_NAME := $(BINARY_NAME)-$(VERSION)-$(OS)-$(ARCH)

.PHONY: help build test build-release package clean

help: ## View this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project with Cargo
	cargo build --all-features --verbose

test: ## Run the tests using Cargo
	cargo test --all-features --verbose -- --nocapture

build-release: ## Build a release version of the project using Cargo
	cargo build --release

package: build-release ## Builds and packages the binaries
	mkdir -p dist
	cp target/release/$(BINARY_NAME) dist/
	tar -czf $(ARCHIVE_NAME).tar.gz -C dist .
	@if [ "$(OS)" = "darwin" ]; then \
		shasum -a 256 $(ARCHIVE_NAME).tar.gz > $(ARCHIVE_NAME).tar.gz.sha256; \
	else \
		sha256sum $(ARCHIVE_NAME).tar.gz > $(ARCHIVE_NAME).tar.gz.sha256; \
	fi

clean: ## Cleans up build artifacts
	cargo clean
	rm -rf dist *.tar.gz *.sha256