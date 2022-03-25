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

find_program() {
  which ${1} >/dev/null 2>&1
  if [ $? -ne 0 ]; then
    return 1
  fi
  return 0
}

install_dependency() {
  find_program apt || return 1
  echo "Installing dependency ${1}"
  if ! apt-get update >/dev/null; then
    echo "Could not install dependency '${1}' (apt update returned error)"
    return 1
  fi
  if ! apt-get -y install ${1} >/dev/null; then
    echo "Could not install dependency '${1}' (apt install returned error)"
    return 1
  fi
}

# ensure running as root
if [ ${EUID} -gt 0 ]; then
  echo "Script has to be run as root" 1>&2
  exit 1
fi

# check and install prerequisites
if ! find_program dpkg; then
  echo "Required program dpkg not found on the system" 1>&2
  exit 1
fi

if ! find_program curl && ! install_dependency curl; then
  exit 1
fi

if ! find_program docker && ! install_dependency docker.io; then
  exit 1
fi

# beta-only: add Docker Hub access token
docker login --username flecs --password $(echo "YzMwMDZmMmYtZWM1My00ZjE5LWEyMjAtYzIyZjZkYjU1OTk1" | base64 -d) >/dev/null 2>&1

# determine dpkg architecture
ARCH=`dpkg --print-architecture`
case ${ARCH} in
  "")
    echo "Could not determine dpkg architecture" 1>&2
    exit 1
    ;;
  "amd64"|"arm64"|"armhf")
    ;;
  *)
    echo "Unsupported architecture ${ARCH} detected, expected amd64|arm64|armhf" 1>&2
    exit 1
    ;;
esac

# determine current version
BASE_URL=https://marketplace.flecs.tech/dl
VERSION=`curl -s -f ${BASE_URL}/latest`
echo "Installing FLECS v${VERSION} for ${ARCH}"

# create temporary directory for download
DOWNLOAD_DIR=`mktemp -d`
if [ $? -ne 0 ]; then
  echo "Could not create temporary directory for download" 1>&2
  exit 1
fi

# download packages
cd ${DOWNLOAD_DIR} || exit 1
PACKAGES=(flecs_${VERSION}_${ARCH}.deb flecs-webapp_${VERSION}_all.deb)
for PACKAGE in ${PACKAGES[*]}; do
  cd ${DOWNLOAD_DIR} || exit 1
  if ! curl -s -f -O ${BASE_URL}/deb/${PACKAGE}; then
    echo "Could not download ${PACKAGE}" 1>&2
    rm -rf ${DOWNLOAD_DIR} >/dev/null 2>&1
    exit 1;
  fi
  cd - >/dev/null 2>&1
done

# install packages
for PACKAGE in ${PACKAGES[*]}; do
  if ! dpkg -i ${DOWNLOAD_DIR}/${PACKAGE} >/dev/null 2>&1; then
    echo "Could not install ${PACKAGE}" 1>&2
    rm -rf ${DOWNLOAD_DIR} >/dev/null 2>&1
    exit 1
  fi
done

echo "Successfully installed FLECS"

# clean up
rm -rf ${DOWNLOAD_DIR} >/dev/null 2>&1
