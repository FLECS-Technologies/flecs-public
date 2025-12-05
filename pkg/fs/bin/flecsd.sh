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

SCRIPTNAME=$(basename $(readlink -f ${0}))

DOCKER_IMAGE=flecspublic.azurecr.io/flecs-slim
DOCKER_TAG=
CONTAINER=flecs-flecsd

print_usage() {
  echo "Usage: ${SCRIPTNAME} <action>"
  echo
  echo "Manage FLECS Core Docker container"
  echo
  echo "Actions:"
  echo "      pull      Pull FLECS Core Docker image"
  echo "      create    Create FLECS Core Docker container"
  echo "      remove    Delete FLECS Core Docker container"
  echo "      stop      Cleanly shutdown FLECS Core Docker container"
  echo "      kill      Kill FLECS Core Docker container"
  echo
}

create_container() {
  NETWORK="--network host"
  VOLUME="--volume /run/docker.sock:/run/docker.sock --volume flecs-floxy_data:/tmp/floxy"
  docker volume create --driver local -o type=tmpfs -o device=tmpfs -o o=size=4m flecs-floxy_data
  docker create \
    --rm \
    --name ${CONTAINER} \
    ${NETWORK} \
    ${VOLUME} \
    --volume flecsd:/var/lib/flecs \
    ${DOCKER_IMAGE}:${DOCKER_TAG}
}

remove_container() {
  docker rm -f ${CONTAINER} >/dev/null 2>&1
}

case ${1} in
  pull)
    # If pulling fails but an image is already present locally,
    # consider pulling successful so the service startup does not fail
    IMAGE_ID=$(docker image ls --quiet ${DOCKER_IMAGE}:${DOCKER_TAG})
    docker pull --quiet ${DOCKER_IMAGE}:${DOCKER_TAG}
    EXIT_CODE=$?
    if [ ${EXIT_CODE} -ne 0 ]; then
      if [ ! -z "${IMAGE_ID}" ]; then
        echo "Using local image ${IMAGE_ID}"
        exit 0
      fi
      exit ${EXIT_CODE}
    fi
    ;;
  create)
    create_container
    exit $?
    ;;
  remove)
    remove_container
    exit $?
    ;;
  stop)
    docker stop --time 120 ${CONTAINER}
    exit $?
    ;;
  kill)
    docker kill --signal KILL ${CONTAINER}
    exit $?
    ;;
  *)
    print_usage
    exit 1
  ;;
esac
