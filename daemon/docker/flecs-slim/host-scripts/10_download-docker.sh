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

DIRNAME=$(dirname $(readlink -f ${0}))

# download .tar.gz archive
cd ${DIRNAME}/../tmp

case ${ARCH} in
  amd64)
    wget https://download.docker.com/linux/static/stable/x86_64/docker-20.10.24.tgz
    ;;
  armhf)
    wget https://download.docker.com/linux/static/stable/armhf/docker-20.10.24.tgz
    ;;
  arm64)
    wget https://download.docker.com/linux/static/stable/aarch64/docker-20.10.24.tgz
    ;;
esac

tar -C ${DIRNAME}/../fs/usr/bin --strip-components=1 -xf docker-*.tgz docker/docker
