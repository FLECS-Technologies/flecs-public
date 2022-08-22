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

DEPENDS="ca-certificates curl gnupg lsb-release net-tools"

run() {
    $* || exit 1;
}

find_program() {
  which ${1} >/dev/null 2>&1
  if [ $? -ne 0 ]; then
    return 1
  fi
  return 0
}

install_program() {
  find_program apt || return 1
  echo "Installing ${1}"
  if ! apt-get update >/dev/null; then
    echo "Could not install '${1}' (apt update returned error)"
    return 1
  fi
  if ! apt-get -y install ${1} >/dev/null; then
    echo "Could not install '${1}' (apt install returned error)"
    return 1
  fi
}

remove_program() {
  echo "Removing ${1}"
  if ! apt-get -y remove ${1} >/dev/null; then
    echo "Could not remove '${1}' (apt remove returned error)"
  fi
}

install_libseccomp () {
  LSB_RELEASE=$(lsb_release -cs)
  if [ "${LSB_RELEASE}" == "buster" ]; then
    echo "deb http://deb.debian.org/debian buster-backports main" > /etc/apt/sources.list.d/00_buster-backports.list
    if ! install_program "libseccomp2/buster-backports"; then
      exit 1;
    fi
  fi
}

install_docker() {
  # remove conflicting packages
  remove_program docker docker-engine docker.io containerd runc

  # get Docker gpg keys
  run curl -fsSL https://download.docker.com/linux/${OS}/gpg | \
    gpg --batch --yes --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

  # add Docker package archive
  run echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/${OS} \
    $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

  # install Docker
  if ! install_program "docker-ce docker-ce-cli containerd.io"; then
    exit 1;
  fi
}

# ensure running as root
if [ ${EUID} -gt 0 ]; then
  echo "Script has to be run as root" 1>&2
  exit 1
fi

# install prerequisites
if ! install_program "${DEPENDS}"; then
  exit 1
fi

# detect OS (Debian or Ubuntu)
OS=$(lsb_release -si | tr '[:upper:]' '[:lower:]')

# workaround for Debian < bullseye
install_libseccomp

# install Docker engine
install_docker

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

# determine latest version
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
PACKAGES=(flecs_${VERSION}_${ARCH}.deb flecs-webapp_${VERSION}_${ARCH}.deb)
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

# clean up
rm -rf ${DOWNLOAD_DIR} >/dev/null 2>&1

echo "Successfully installed FLECS"
