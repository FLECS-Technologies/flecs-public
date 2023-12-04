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

project(flecsd)

add_subdirectory(api)
add_subdirectory(common)
add_subdirectory(modules)

# build executable flecsd
add_executable(daemon
    src/main.cpp
)
set_target_properties(daemon PROPERTIES OUTPUT_NAME flecsd)

# own includes
target_include_directories(daemon PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# FLECS link libraries
target_link_libraries(daemon PRIVATE
    daemon.api
    daemon.common
    daemon.modules.apps
    daemon.modules.data_layer
    daemon.modules.device
    daemon.modules.factory
    daemon.modules.flecsport
    daemon.modules.instances
    daemon.modules.jobs
    daemon.modules.manifests
    daemon.modules.marketplace
    daemon.modules.module_base
    daemon.modules.system
    daemon.modules.version
    util.datetime
    util.process
    util.signal_handler
    yaml-cpp::yaml-cpp
)

# install executable flecsd for packaging
install(
    TARGETS daemon RUNTIME
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/opt/flecs/bin
)

# install shared objects for packaging
install(
    TARGETS daemon.lib flunder.shared LIBRARY
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/opt/flecs/lib
)

# install helper scripts for packaging
install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/scripts/
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/opt/flecs/bin
    USE_SOURCE_PERMISSIONS
)

# configure Docker image flecs
set(DOCKER_REGISTRY "cr.flecs.tech")
set(DOCKER_IMAGE "flecs")
set(DOCKER_TAG "${CORE_VERSION}")
include(flecs-docker)

# configure Docker image flecs-slim
set(DOCKER_REGISTRY "cr.flecs.tech")
set(DOCKER_IMAGE "flecs-slim")
set(DOCKER_TAG "${CORE_VERSION}")
include(flecs-docker)

# configure package
set(PACKAGE flecs)
set(PACKAGE_VERSION ${CORE_VERSION})
set(PACKAGE_DESC FLECS)
set(PACKAGE_DEPENDS "docker-ce (>= 20.10.5) | docker.io (>= 20.10.5)")

add_custom_target(
    flecs_deb-pkg-copy
    WORKING_DIRECTORY ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/pkg
    COMMAND pwd
    COMMAND sed -i 's/^DOCKER_TAG.*/DOCKER_TAG=${DOCKER_TAG}/g' */opt/flecsd/bin/flecsd.sh
)

include(flecs-package)