#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}
image=${2:?Please provide the image name for the cassandra database}

source "$(dirname $0)/../lib/utilities.sh"

# More verbosity on travis for now
if [ -n "$TRVIS" ]; then
    set -x
fi

set -eu
port=$CASSANDRA_PORT
host=$CASSANDRA_HOST
set +u

trap stop-dependencies 0 1 2 5 15
# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: PLAIN"
start-dependencies-plain $image
$cli test-connection $host $port

# TODO plain + TLS

# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-AUTHENTICATION"
start-dependencies-auth $image
$cli test-connection -u cassandra -p cassandra $host $port

# TODO auth + TLS

echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-CERTIFICATE"
start-dependencies-tls $image
# TODO: provide certificate, possibly test interaction with auth ... might just not be needed though.
$cli test-connection $host

