#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=${SCRIPT_DIR}/..

source ${ROOT_DIR}/docker/env

if [ ! $( docker ps -a | grep ${CNAMETSRV} | wc -l ) -gt 0 ]; then
  docker run -dit \
    -v ${ROOT_DIR}:/recmd \
    -e CARGO_TARGET_DIR=/recmd/target_docker \
    --name ${CNAMETSRV} \
    ${CTAGT}
fi

if [ ! $( docker ps -a | grep ${CNAMETSND} | wc -l ) -gt 0 ]; then
  docker run -dit \
    -v ${ROOT_DIR}:/recmd \
    -e CARGO_TARGET_DIR=/recmd/target_docker \
    --name ${CNAMETSND} \
    ${CTAGT}
fi

docker exec -it \
  ${CNAMETSRV} \
  cargo build --release
docker exec -it \
  ${CNAMETSND} \
  cargo build --release

SRVIP=`docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' testsrvrecmdbuild`

echo "### Run server with:"
echo "docker exec -it testsrvrecmdbuild target_docker/release/recmd srv -p 2222 -d"
echo "### Run client with:"
echo "docker exec -it testsndrecmdbuild target_docker/release/recmd snd -i ${SRVIP} -p 2222 -c 'ls'"
