test:
	@pgrep testserver | xargs kill;
	@cargo clippy
	@cargo build --package testserver
	@./target/debug/testserver &
	@cargo nextest run --lib --config-file nextest.conf
	@pgrep testserver | xargs kill;

test-verbose:
	@pgrep testserver | xargs kill;
	@cargo clippy
	@cargo build --package testserver
	@./target/debug/testserver &
	@cargo nextest run --config-file nextest.conf --no-capture
	@pgrep testserver | xargs kill;
