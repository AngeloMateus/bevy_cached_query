TESTS =

test:
	@pgrep testserver | xargs kill;
	@cargo clippy
	@cargo build --package testserver
	@./target/debug/testserver &
	@cargo nextest run --lib --config-file nextest.conf
	@pgrep testserver | xargs kill;

test-verbose:
	echo $(TESTS)
	@pgrep testserver | xargs kill;
	@cargo clippy
	@cargo build --package testserver
	@./target/debug/testserver &
	cargo nextest run --config-file nextest.conf --no-capture -- $(TESTS)
	@pgrep testserver | xargs kill;
