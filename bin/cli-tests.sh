#!/usr/bin/env bash

cli=${1:?Please provide the commandline interface executable as first argument}

conargs="-h localhost"
query="query --dry-run"

$cli $conargs $query \
  && { echo "should fail if no query parameter was provided at all for now"; exit 1; }
  
[ "$($cli $conargs $query -e foo)" = "foo;" ] \
  || { echo "--execute must become part of query and end in semicolon"; exit 2; }
  
[ "$($cli $conargs $query -e foo\;)" = "foo;" ] \
  || { echo "There are no double-semicolons"; exit 2; }
  
[ "$($cli $conargs $query -k ks -e foo)" = "use ks; foo;" ] \
  || { echo "a keyspace is prepended"; exit 2; }
  
[ "$($cli $conargs $query -k ks)" = "use ks;" ] \
  || { echo "Just the keyspace is fine"; exit 2; }

echo OK  
