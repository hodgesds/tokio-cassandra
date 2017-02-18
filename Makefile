CLI_EXECUTABLE=target/debug/tcc

help:
	$(info Available Targets)
	$(info --------------------------)
	$(info toc                  | generate table of contents for README.md via doctoc)
	$(info unit-tests           | Run tests that don't need a cassandra node running)
	$(info integration-tests    | Run tests that use a cassandra node)
	$(info debug-cli-tests      | Run the cli with certain arguments to help debugging - needs debug-docker-db)
	$(info debug-docker-db      | Bring up a cassandra database for local usage on 9042)

toc:
	doctoc --github --title "A Cassandra Native Protocol 3 implementation using Tokio for IO." README.md
	
unit-tests:
	cargo doc
	cargo test --all-features

$(CLI_EXECUTABLE):
	cd cli && cargo build

integration-tests: $(CLI_EXECUTABLE)
	bin/integration-test.sh $(CLI_EXECUTABLE)

debug-docker-db: $(CLI_EXECUTABLE)
	. lib/utilities.sh && start_dependencies 9042 "$(CLI_EXECUTABLE) test-connection"

debug-cli-tests:
	cd cli && cargo run -- test-connection 127.0.0.1 9042


