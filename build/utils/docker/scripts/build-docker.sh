## Copyright 2021-2022 FLECS Technologies GmbH
##
## Licensed under the Apache License, Version 2.0 (the "License");
## you may not use this file except in compliance with the License.
## You may obtain a copy of the License at
##
## http://www.apache.org/licenses/LICENSE-2.0
##
## Unless required by applicable law or agreed to in writing, software
## distributed under the License is distributed on an "AS IS" BASIS,
## WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
## See the License for the specific language governing permissions and
## limitations under the License.

#!/bin/bash

[ -z "${1}" ] && exit 1

IMAGE=$1
shift

for arg in "$@"; do
    DOCKER_ARGS="${DOCKER_ARGS} --build-arg $arg"
done

DIRNAME=$(readlink -f $(pwd))/${IMAGE}

echo "Building image ${IMAGE} in context ${DIRNAME}"
mkdir -p ${DIRNAME}/build-utils
cp -r $(git rev-parse --show-toplevel)/build-utils/docker ${DIRNAME}/build-utils/
docker build -t flecs/${IMAGE} -f ${DIRNAME}/dockerfiles/Dockerfile.${IMAGE} ${DOCKER_ARGS} ${DIRNAME}
rm -rf ${DIRNAME}/build-utils
