CLI_EXECUTABLE=target/debug/tcc
DB_IMAGE_OK=.db-image.ok
DB_IMAGE_NAME=our/cassandra:latest
MAKESHELL=$(shell /usr/bin/env bash)

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info toc              | generate table of contents for README.md via doctoc)
	$(info unit-tests       | Run tests that don't need a cassandra node running)
	$(info integration-tests| Run tests that use a cassandra node)
	$(info -- DEBUGGING --------------------------------------------------------------------------------------------------)
	$(info cli-tests        | Run the cli with certain arguments to help debugging - needs debug-<some>-docker-db)
	$(info plain-docker-db  | Bring up a backgrounded cassandra database for local usage on 9042)
	$(info auth-docker-db   | Bring up a backgrounded cassandra database for local usage on 9042, requiring authentication)
	$(info tls-docker-db    | Bring up a backgrounded cassandra database for local usage on 9042, requiring TLS)
	$(info attach-docker-db | run cassandra in foreground run with type=(tls|auth|plain))

toc:
	doctoc --github --title "A Cassandra Native Protocol 3 implementation using Tokio for IO." README.md
	
unit-tests:
	cargo doc
	cargo test --all-features

$(CLI_EXECUTABLE): $(shell find cli -name "*.rs")
	cd cli && cargo build

integration-tests: $(CLI_EXECUTABLE) $(DB_IMAGE_OK)
	bin/integration-test.sh $(CLI_EXECUTABLE) $(DB_IMAGE_NAME)

auth-docker-db: $(CLI_EXECUTABLE) $(DB_IMAGE_OK)
	/usr/bin/env bash -c 'source lib/utilities.sh && start_dependencies $(DB_IMAGE_NAME) 9042 "does not matter" "-e CASSANDRA_AUTHENTICATOR=PasswordAuthenticator"'

tls-docker-db: $(CLI_EXECUTABLE) $(DB_IMAGE_OK)
	/usr/bin/env bash -c 'source lib/utilities.sh && start_dependencies $(DB_IMAGE_NAME) 9042 "does not matter" "-e CASSANDRA_REQUIRE_CLIENT_AUTH=true"'

plain-docker-db: $(CLI_EXECUTABLE) $(DB_IMAGE_OK)
	/usr/bin/env bash -c 'source lib/utilities.sh && start_dependencies $(DB_IMAGE_NAME) 9042 "does not matter"'

type ?= plain
attach-docker-db:
	DEBUG_RUN_IMAGE=true $(MAKE) debug-$(type)-docker-db

cli-tests:
	cd cli && cargo run -- test-connection 127.0.0.1 9042

$(DB_IMAGE_OK): $(shell find etc/docker-cassandra -type f) bin/build-image.sh
	bin/build-image.sh etc/docker-cassandra $(DB_IMAGE_NAME)
	@touch $(DB_IMAGE_OK)

