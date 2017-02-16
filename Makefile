
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
	bin/integration-test.sh cli/target/debug/tcc

# docker run --rm --name cassandra-test -d -p 9042:9042 --expose 9042 cassandra:2.1
