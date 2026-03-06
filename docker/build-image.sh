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

SCRIPT_DIR=$(dirname $(readlink -f ${0}))
BUILD_DIR=$(readlink -f $(pwd))

while [ ! -z "${1}" ]; do
  case $1 in
    --arch)
      ARCH="${2}"
      shift
      shift
      ;;
    --docker-arg)
      DOCKER_ARGS+=("${2}")
      shift
      shift
      ;;
    --file)
      DOCKERFILE="${2}"
      shift
      shift
      ;;
    --image)
      IMAGE="$2"
      shift
      shift
      ;;
    --platform)
      PLATFORM="$2"
      shift
      shift
      ;;
    --tag)
      DOCKER_TAG="${2}"
      shift
      shift
      ;;
    --variant)
      VARIANT="${2}"
      shift
      shift
      ;;
    *)
      echo "Unknown option $1"
      exit 1
      ;;
  esac
done

if [ -z "${IMAGE}" ]; then
  echo "No IMAGE specified" 1>&2
  exit 1
fi

if [ -z "${ARCH}" ] && [ -z "${PLATFORM}" ]; then
  echo "Neither ARCH nor PLATFORM specified" 1>&2
  exit 1
fi

if [ -z "${ARCH}" ]; then
  case ${PLATFORM} in
  linux/arm/v7)
    ARCH=armhf
    ;;
  linux/amd64)
    ARCH=amd64
    ;;
  linux/arm64)
    ARCH=arm64
    ;;
  *)
    echo "Invalid platform ${PLATFORM} specified" 1>&2
    exit 1
    ;;
  esac
fi

if [ -z "${PLATFORM}" ]; then
  case ${ARCH} in
  amd64)
    PLATFORM="linux/amd64"
    ;;
  arm64)
    PLATFORM="linux/arm64"
    ;;
  armhf)
    PLATFORM="linux/arm/v7"
    ;;
  *)
    echo "Invalid architecture ${ARCH} specified" 1>&2
    exit 1
    ;;
  esac
fi

case ${ARCH} in
amd64|arm64|armhf)
  ;;
*)
  echo "Invalid architecture ${ARCH} specified" 1>&2
  exit 1
  ;;
esac

case ${PLATFORM} in
linux/arm/v7|linux/amd64|linux/arm64)
  ;;
*)
  echo "Invalid platform ${PLATFORM} specified" 1>&2
  exit 1
  ;;
esac

if [ -z "${DOCKER_TAG}" ]; then
  DOCKER_TAG="latest"
fi

if [ -z "${DOCKERFILE}" ]; then
  if [ -d "${BUILD_DIR}/${IMAGE##*/}${VARIANT}" ] && [ -f "${BUILD_DIR}/${IMAGE##*/}${VARIANT}/Dockerfile" ]; then
    DOCKERFILE="${BUILD_DIR}/${IMAGE##*/}${VARIANT}/Dockerfile"
  elif [ -f "${BUILD_DIR}/Dockerfile" ]; then
    DOCKERFILE="${BUILD_DIR}/Dockerfile"
  else
    echo "No DOCKERFILE specified and none found" 1>&2
    exit 1
  fi
fi

export ARCH

echo "Building image ${IMAGE}:${DOCKER_TAG}-${ARCH} in context ${BUILD_DIR}"

if [ -d ${BUILD_DIR}/host-scripts ]; then
  cd ${BUILD_DIR}/host-scripts;
  for sh in *.sh; do
    echo "Running host script ${sh}"
    bash -x "${sh}" || exit 1
  done
fi

if [ -d ${BUILD_DIR}/${IMAGE}${VARIANT}/host-scripts ]; then
  cd ${BUILD_DIR}/${IMAGE}${VARIANT}/host-scripts;
  for sh in *.sh; do
    echo "Running host script ${sh}"
    bash -x "${sh}" || exit 1
  done
fi

docker buildx build \
  --load \
  --build-arg ARCH=${ARCH} \
  --platform ${PLATFORM} \
  --tag ${IMAGE}:${DOCKER_TAG}-${ARCH} \
  --file ${DOCKERFILE} \
  ${DOCKER_ARGS} ${BUILD_DIR};
