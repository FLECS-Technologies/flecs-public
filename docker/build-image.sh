#!/bin/bash

[ -z "${1}" ] && exit 1

IMAGE=$1
shift

for arg in "$@"; do
    DOCKER_ARGS="${DOCKER_ARGS} --build-arg $arg"
done

DIRNAME=$(dirname $(readlink -f ${0}))/${IMAGE}

echo "Building image ${IMAGE} in context ${DIRNAME}"
mkdir -p ${DIRNAME}/build-utils
cp -r $(git rev-parse --show-toplevel)/build-utils/docker ${DIRNAME}/build-utils/
docker buildx build \
	--tag flecs/${IMAGE} \
	--platform linux/amd64,linux/arm/v7,linux/arm64 \
	--output=type=registry \
	--file ${DIRNAME}/dockerfiles/Dockerfile.${IMAGE} ${DOCKER_ARGS} ${DIRNAME}
rm -rf ${DIRNAME}/build-utils
