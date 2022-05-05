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

run() {
    $* || exit 1;
}

# determine OS name (debian, ubuntu, ...)
OS=$(lsb_release -si | tr '[:upper:]' '[:lower:]')

# get Docker gpg keys
run curl -fsSL https://download.docker.com/linux/${OS}/gpg | \
  gpg --batch --yes --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# add Docker package archive
run echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/${OS} \
  $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

# install Docker
run apt-get -y update
run apt-get -y install docker-ce docker-ce-cli containerd.io
