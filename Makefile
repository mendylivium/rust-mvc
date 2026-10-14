.PHONY: install dev build test check migrate seed
install:
	cargo fetch
	npm ci
dev:
	node scripts/dev.mjs
build:
	npm run build
	cargo build --release
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
