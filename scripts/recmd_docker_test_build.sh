#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=${SCRIPT_DIR}/..

source ${ROOT_DIR}/docker/env

docker rm ${CNAMETSRV}
docker rm ${CNAMETSND}
docker build -t ${CTAGT} ${ROOT_DIR}/docker/test
