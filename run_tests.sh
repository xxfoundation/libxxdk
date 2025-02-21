#!/usr/bin/env bash

rm -rf ignore.*

exitcode=0
while [ ! -z "$1" ]; do
  echo Running test suite $1...
  ./"$1" || exitcode=1
  shift
done
exit $exitcode
