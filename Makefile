.PHONY: release
release:
	cargo build --release

.PHONY: dev
dev: format lint test doc

.PHONY: d
d:
	cargo watch -c -s 'make dev'

.PHONY: lint
lint:
	cargo clippy --all-targets

.PHONY: test
test:
	cargo test --all-targets

.PHONY: format
format:
	cargo fmt

.PHONY: doc
doc:
	cargo doc
