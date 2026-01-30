# Copyright 2021-2024 FLECS Technologies GmbH
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

install(PROGRAMS fs/entrypoint.sh
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs
    COMPONENT docker
    EXCLUDE_FROM_ALL
)
install(FILES Dockerfile
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}
    COMPONENT docker
    EXCLUDE_FROM_ALL
)

stage_docker_fs()

set(DOCKER_REGISTRY "flecspublic.azurecr.io")
set(DOCKER_IMAGE "flecs")
set(DOCKER_TAG "${FLECS_VERSION_FULL}")
include(flecs-docker)
