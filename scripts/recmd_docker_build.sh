#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=${SCRIPT_DIR}/..

source ${ROOT_DIR}/docker/env

if [ ! $( docker ps -a | grep ${CNAME} | wc -l ) -gt 0 ]; then
  docker run -dit \
    -v ${ROOT_DIR}:/recmd \
    -e CARGO_TARGET_DIR=/recmd/target_docker \
    --name ${CNAME} \
    ${CTAG}
fi

if [ -z "$1" ] ; then
  docker exec -it \
    ${CNAME} \
    cargo build --release
else
  docker exec -it \
    ${CNAME} \
    $@
fi
