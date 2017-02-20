#!/bin/bash

set -e

dc=${1:?First argument must be the clone of the docker-cassandra repository}
image_name=${2:?Second argument must be the image name for the cassandra database}
mkdir -p "$(dirname "$dockerfile")"

cd "$dc"
./build.sh -v 2.1 -t "$image_name"