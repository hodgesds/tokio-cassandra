
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
	
integration-tests:
	cd cli && cargo build
	bin/integration-test.sh target/debug/tcc

debug-docker-db:
	source lib/utilities.sh && start_dependencies 9042

debug-cli-tests:
	cd cli && cargo run -- test-connection 127.0.0.1 9042


