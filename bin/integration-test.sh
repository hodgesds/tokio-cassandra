#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}

set -eu

CASSANDRA_HOST=127.0.0.1
CASSANDRA_PORT=9098

function start_dependencies() {
    echo starting dependencies
}

function stop_dependencies() {
    echo stopping dependencies
}

res=start_dependencies
trap "stop_dependencies $res" 0 1 2 5 15


$cli test-connection $CASSANDRA_HOST $CASSANDRA_PORT

