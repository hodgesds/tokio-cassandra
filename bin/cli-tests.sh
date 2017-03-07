#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}

conargs="-h localhost"
query="query --dry-run"

set -x
$cli $conargs $query \
  && { echo "should fail if no query parameter was provided at all for now"; exit 1; }
  
set +x
[ "$($cli $conargs $query -e foo)" = foo ] \
  || { echo "--execute must become part of query"; exit 2; }
  
echo OK  
