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

DIR=$(dirname $(readlink -f ${0}))

BUILD_DIR='build/${ARCH}'
INSTALL_DIR='out/${ARCH}'

while [ "$1" != "" ]; do
  case $1 in
    --debug)
      BUILD_TYPE=Debug
      ;;
    --release)
      BUILD_TYPE=Release
      ;;
    --with-tests)
      CMAKE_OPTIONS+="-DFLECS_BUILD_TESTS=On"
      BUILD_TYPE=RelWithDebInfo
      BUILD_DIR=build/test
      INSTALL_DIR=out/test
      ;;
    --arch)
      shift
      case $1 in
        armhf|arm-linux-gnueabihf)
          ARCH=armhf
          ;;
        aarch64|arm64|aarch64-linux-gnu)
          ARCH=arm64
          ;;
        amd64|x86_64|x64|x86_64-linux-gnu)
          ARCH=amd64
          ;;
        *)
          echo "Unknown architecture $1" 1>&2
          exit 1
          ;;
    esac

  esac
  shift
done

if [ "${BUILD_TYPE}" = "" ]; then
  BUILD_TYPE=Debug
fi

if [ "${ARCH}" = "" ]; then
  ARCH=amd64
fi

echo "Building ${BUILD_TYPE} for ${ARCH} with options ${CMAKE_OPTIONS}"

BUILD_DIR=`eval echo ${BUILD_DIR}`
INSTALL_DIR=`eval echo ${INSTALL_DIR}`

cmake -G Ninja -B ${BUILD_DIR} -DARCH=${ARCH} -DCMAKE_BUILD_TYPE=${BUILD_TYPE} -DCMAKE_INSTALL_PREFIX=${INSTALL_DIR} ${CMAKE_OPTIONS}
cmake --build ${BUILD_DIR}
