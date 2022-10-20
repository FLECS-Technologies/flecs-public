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

DIRNAME=$(dirname $(readlink -f ${0}))

curl -fsSL https://download.docker.com/linux/debian/gpg |\
  gpg --dearmor -o ${DIRNAME}/../fs/usr/share/keyrings/docker-archive-keyring.gpg

echo \
  "deb [arch=${ARCH} signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] \
  https://download.docker.com/linux/debian bullseye stable" \
  >${DIRNAME}/../fs/etc/apt/sources.list.d/docker.list
