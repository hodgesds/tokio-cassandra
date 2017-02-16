
help:
	$(info Available Targets)
	$(info --------------------------)
	$(info unit-tests           | Run tests that don't need a cassandra node running)
	$(info integration-tests    | Run tests that use a cassandra node)
	
unit-tests:
	cargo doc
	cargo test --all-features
	
integration-tests:
	cd cli && cargo build
	bin/integration-test.sh target/debug/tcc
