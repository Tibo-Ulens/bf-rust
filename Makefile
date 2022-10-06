.PHONY: br fmt lint

br: fmt
	cargo +nightly build --release

fmt:
	cargo +nightly fmt

lint:
	cargo +nightly clippy
