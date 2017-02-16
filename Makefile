CLI_EXECUTABLE=target/debug/tcc

help:
	$(info Available Targets)
	$(info --------------------------)
	$(info unit-tests           | Run tests that don't need a cassandra node running)
	$(info integration-tests    | Run tests that use a cassandra node)
	$(info debug-cli-tests      | Run the cli with certain arguments to help debugging - needs debug-docker-db)
	$(info debug-docker-db      | Bring up a cassandra database for local usage on 9042)
	
unit-tests:
	cargo doc
	cargo test --all-features

$(CLI_EXECUTABLE):
	cd cli && cargo build

integration-tests: 
	bin/integration-test.sh $(CLI_EXECUTABLE)

debug-docker-db: $(CLI_EXECUTABLE)
	. lib/utilities.sh && start_dependencies 9042 "$(CLI_EXECUTABLE) test-connection"

debug-cli-tests:
	cd cli && cargo run -- test-connection 127.0.0.1 9042


