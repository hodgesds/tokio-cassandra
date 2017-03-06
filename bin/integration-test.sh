#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}
image=${2:?Please provide the image name for the cassandra database}

source "$(dirname $0)/../lib/utilities.sh"

# More verbosity on travis for now
[ -n "$TRAVIS" ] && set -x

set -eu
port=$CASSANDRA_PORT
host=$CASSANDRA_HOST_NAME
ip=$CASSANDRA_HOST_IP
set +u

ca_file=./etc/docker-cassandra/secrets/keystore.cer.pem
con_ip_args="-h $ip --port $port"
con_host_args="-h $host --port $port"

#########################################################################
echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: PLAIN           <<<<<<<<<<<<<"
#########################################################################
start-dependencies-plain $image

set -x
$cli $con_ip_args test-connection
$cli $con_host_args test-connection

$cli $con_ip_args --tls --ca-file $ca_file  test-connection \
  && { echo "should not connect if ip is set when using tls - verification must fail"; exit 1; }
$cli $con_host_args --tls --ca-file $ca_file  test-connection 
$cli $con_host_args --ca-file $ca_file  test-connection \
  || { echo "should imply tls if CA-file is specified"; exit 2; }
$cli $con_host_args --tls test-connection \
  && { echo "should fail TLS hostname verification on self-signed cert by default"; exit 3; }
set +x

#########################################################################
echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-AUTHENTICATION <<<<<<<<"
#########################################################################
trap stop-dependencies 0 1 2 5 15
start-dependencies-auth $image
# YES - there is something async going on, so we have to give it even more time until 
# it can accept properly authenticated connections
sleep 1

auth_args="-u cassandra -p cassandra" 
tls_with_trust="--ca-file $ca_file"

set -x
$cli $auth_args $con_ip_args test-connection
$cli $auth_args $con_host_args $tls_with_trust test-connection
set +x


# TODO auth + TLS

# echo ">>>>>>>>>>>>>>>>>>>> TEST CONNECTION: WITH-CERTIFICATE"
start-dependencies-cert $image
# TODO: provide certificate, possibly test interaction with auth ... might just not be needed though.
# $cli test-connection $host

