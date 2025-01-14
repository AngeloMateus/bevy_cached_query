test:
	cargo clippy
	cargo nextest run --lib --config-file nextest.conf

test-verbose:
	cargo nextest run --lib --config-file nextest.conf -- --show-output
