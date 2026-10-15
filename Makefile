MODE ?= server
TARGET ?= $(shell node -p "({win32:'windows',darwin:'macos',linux:'linux'})[process.platform]")
ARCH ?= $(shell node -p "process.arch")

.PHONY: install dev build test check migrate seed
install:
	cargo fetch
	npm ci
dev:
	node scripts/dev.mjs
build:
	node scripts/build.mjs --$(MODE) --target $(TARGET) --arch $(ARCH)
test:
	cargo test
check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	npm run build
migrate:
	./forge migrate
seed:
	./forge db:seed
