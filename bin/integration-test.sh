#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}
image=${2:?Please provide the image name for the cassandra database}

source "$(dirname $0)/../lib/utilities.sh"

# More verbosity on travis for now
[ -n "$TRAVIS" ] && set -x

set -eu
port=$CASSANDRA_PORT
host=$CASSANDRA_HOST
set +u

# TODO plain + TLS

# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-AUTHENTICATION"
trap stop-dependencies 0 1 2 5 15
start-dependencies-auth $image
# YES - there is something async going on, so we have to give it even more time until 
# it can accept properly authenticated connections
sleep 1
$cli -u cassandra -p cassandra -h $host --port $port test-connection 

# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: PLAIN"
start-dependencies-plain $image
$cli -h $host --port $port test-connection

# TODO auth + TLS

# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-CERTIFICATE"
start-dependencies-cert $image
# TODO: provide certificate, possibly test interaction with auth ... might just not be needed though.
# $cli test-connection $host

