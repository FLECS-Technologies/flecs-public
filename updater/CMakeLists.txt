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

project(FLECS.updater)

# configure package
set(PACKAGE flecs-updater)
set(PACKAGE_DESC FLECS)

include(package)

# configure Docker image
set(DOCKER_IMAGE "updater")

add_custom_target(
    ${DOCKER_IMAGE}_prepare
    COMMAND mkdir -p ${CMAKE_CURRENT_SOURCE_DIR}/out/${MACHINE}/opt/flecs/pkg
    COMMAND tar czf ${CMAKE_CURRENT_SOURCE_DIR}/out/${MACHINE}/opt/flecs/pkg/flecs.deb.tar.gz -C ${CMAKE_BINARY_DIR}/ flecs_${VERSION}_${ARCH}.deb
    COMMAND tar czf ${CMAKE_CURRENT_SOURCE_DIR}/out/${MACHINE}/opt/flecs/pkg/flecs-updater.deb.tar.gz -C ${CMAKE_BINARY_DIR}/ flecs-updater_${VERSION}_${ARCH}.deb
)

include(docker)
