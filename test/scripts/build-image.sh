#!/bin/bash

# Copyright 2021-2023 FLECS Technologies GmbH
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

if [ ${EUID} -gt 0 ]; then
  echo "Script has to be run as root" 1>&2
  exit 1
fi

for arg in "$@"; do
  DOCKER_ARGS="${DOCKER_ARGS} --build-arg $arg"
done

DIRNAME=$(dirname $(readlink -f ${0}))/..

docker buildx build \
    --load \
    --build-arg ARCH=amd64 \
    --tag "flecs/flecs-test:latest" \
    --file dockerfiles/Dockerfile.Debian-11 \
    ${DOCKER_ARGS} ${DIRNAME}


docker run \
    --name flecs-test -it --rm --privileged \
    --env-file ./env.test \
    flecs/flecs-test:latest
