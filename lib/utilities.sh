#!/bin/bash
CONTAINER_NAME=db
CASSANDRA_HOST=127.0.0.1

read -r -d '' ENV_FILE <<EOF
CASSANDRA_ENABLE_SSL=false
	CASSANDRA_ENABLE_SSL_DEBUG=true
	CASSANDRA_KEYSTORE_PASSWORD=cassandra
	CASSANDRA_TRUSTSTORE_PASSWORD=cassandra
	CASSANDRA_SSL_PROTOCOL=TLS
	CASSANDRA_SSL_ALGORITHM=SunX509
EOF

start_dependencies() {
	local IMAGE_NAME=${1:?Need cassandra image name}
	local CASSANDRA_PORT=${2:?Need cassandra port to expose/expect on host}
	local TESTER=${3:?Need command line to execute to see if cassandra is up}
	echo starting dependencies
	local debug_mode=${DEBUG_RUN_IMAGE:-false}
	local daemonize="-d"
	if [ "$debug_mode" = true ]; then
		daemonize=''
	fi
	docker rm --force $CONTAINER_NAME || true;
	docker run --name "$CONTAINER_NAME" --env-file <(echo "$ENV_FILE") $daemonize -p "$CASSANDRA_HOST":"$CASSANDRA_PORT":9042 --expose 9042 $IMAGE_NAME 1>&2 || exit $?
	
	if [ "$debug_mode" = false ]; then
		while ! $TESTER "$CASSANDRA_HOST" "$CASSANDRA_PORT"; do
			echo "Waiting for cassandra on $CASSANDRA_HOST:$CASSANDRA_PORT" 1>&2
			sleep 1
		done
		echo "Cassandra up on $CASSANDRA_HOST:$CASSANDRA_PORT" 1>&2
	fi
}

stop_dependencies() {
	echo stopping dependencies ...
	docker rm --force $CONTAINER_NAME >/dev/null
}
