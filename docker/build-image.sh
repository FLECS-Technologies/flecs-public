#!/bin/bash

# Copyright 2021-2022 FLECS Technologies GmbH
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

[ -z "${1}" ] && exit 1

IMAGE=$1
shift

for arg in "$@"; do
  DOCKER_ARGS="${DOCKER_ARGS} --build-arg $arg"
done

DIRNAME=$(dirname $(readlink -f ${0}))/${IMAGE}

echo "Building image ${IMAGE} in context ${DIRNAME}"

for PLATFORM in $(cat ${DIRNAME}/${IMAGE}.platforms); do
  mkdir -p ${DIRNAME}/build-utils
  cp -r $(git rev-parse --show-toplevel)/build/utils/docker ${DIRNAME}/build-utils/

  case ${PLATFORM} in
  linux/arm/v7)
    export ARCH=armhf
    ;;
  linux/amd64)
    export ARCH=amd64
    ;;
  linux/arm64)
    export ARCH=arm64
    ;;
  *)
    echo "Invalid platform ${PLATFORM}" 1>&2
    exit 1
    ;;
  esac
  if [ -d ${DIRNAME}/host-scripts ]; then
  	cd ${DIRNAME}/host-scripts;
  	for sh in *.sh; do
  	  bash "${sh}"
  	done
  fi

  docker buildx build \
    --load \
    --build-arg ARCH=${ARCH} \
    --platform ${PLATFORM} \
    --tag flecs/${IMAGE}:latest-${ARCH} \
    --file ${DIRNAME}/dockerfiles/Dockerfile.${IMAGE} \
    ${DOCKER_ARGS} ${DIRNAME};

  git clean -dxf -- ${DIRNAME}
done
