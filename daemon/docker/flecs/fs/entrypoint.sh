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

PATH=/sbin:/usr/sbin:/bin:/usr/bin:/opt/flecs/bin

containerd >/tmp/containerd.log 2>&1 &
dockerd >/tmp/dockerd.log 2>&1 &

# wait for Docker daemon to be ready
while ! docker version >/dev/null 2>&1; do
    sleep 1
done

exec flecsd
