
lint:
	cargo clippy --all-targets --all-features -- -D warnings
test:
	cargo test
test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --test-threads=1 --nocapture
