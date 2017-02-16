

CONTAINER_NAME=db
CASSANDRA_HOST=127.0.0.1

start_dependencies() {
	CASSANDRA_PORT=${1:?Need cassandra port to expose/expect on host}
	TESTER=${2:?Need command line to execute to see if cassandra is up}
	echo starting dependencies
	docker run --name "$CONTAINER_NAME" -d -p "$CASSANDRA_HOST":"$CASSANDRA_PORT":9042 --expose 9042 cassandra:2.1 1>&2
	while ! $TESTER "$CASSANDRA_HOST" "$CASSANDRA_PORT"; do
		echo "Waiting for cassandra on $CASSANDRA_HOST:$CASSANDRA_PORT" 1>&2
		sleep 1
	done
	echo "Cassandra up on $CASSANDRA_HOST:$CASSANDRA_PORT" 1>&2
}

stop_dependencies() {
	echo stopping dependencies ...
	docker rm --force $CONTAINER_NAME >/dev/null
}
