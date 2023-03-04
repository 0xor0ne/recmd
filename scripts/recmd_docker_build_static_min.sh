#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=${SCRIPT_DIR}/..

source ${ROOT_DIR}/docker/env

if [ ! $( docker ps -a | grep ${CNAMES} | wc -l ) -gt 0 ]; then
  docker run -dit \
    -v ${ROOT_DIR}:/recmd \
    -e CARGO_TARGET_DIR=/recmd/target_docker_static \
    --name ${CNAMES} \
    ${CTAGS}
fi

if [ -z "$1" ] ; then
  docker exec -it \
    ${CNAMES} \
    cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-musl --release
else
  docker exec -it \
    ${CNAMES} \
    $@
fi
