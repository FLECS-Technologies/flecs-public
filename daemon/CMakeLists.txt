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

# configure package
set(PACKAGE flecs)
set(PACKAGE_VERSION ${CORE_VERSION})
set(PACKAGE_DESC FLECS)
set(PACKAGE_DEPENDS "net-tools, docker-ce (>= 20.10.5) | docker.io (>= 20.10.5), libudev")

include(flecs-package)

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
    external.yaml-cpp
)

# install executable flecsd for packaging
install(
    TARGETS daemon RUNTIME
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/bin
)
install(
    TARGETS daemon RUNTIME
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/bin
)

# install shared objects for packaging
install(
    TARGETS daemon.lib mqtt.shared LIBRARY
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/lib
)

install(
    TARGETS daemon.lib flunder.shared LIBRARY
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/lib
)

install(
    FILES $<TARGET_FILE:external.zenoh-c>
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/lib
)
install(
    TARGETS daemon.lib mqtt.shared LIBRARY
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/lib
)

install(
    TARGETS daemon.lib flunder.shared LIBRARY
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/lib
)

install(
    FILES $<TARGET_FILE:external.zenoh-c>
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/lib
)

# install FLECS cli for packaging
install(
    TARGETS cli RUNTIME
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/bin
)
install(
    TARGETS cli RUNTIME
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/bin
)

# install hosts script for packaging
install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/scripts/flecs-update-hosts.sh
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/bin
)
install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/scripts/flecs-update-hosts.sh
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/opt/flecs/bin
)

# install Docker images
get_target_property(DOCKER_ARCHIVE tech.flecs.mqtt-bridge_docker DOCKER_ARCHIVE)
get_filename_component(DOCKER_ARCHIVE ${DOCKER_ARCHIVE} NAME)
string(REPLACE _${ARCH} "" DOCKER_ARCHIVE ${DOCKER_ARCHIVE})
install(
    FILES $<TARGET_PROPERTY:tech.flecs.mqtt-bridge_docker,DOCKER_ARCHIVE>
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/assets
    RENAME ${DOCKER_ARCHIVE}
)
get_target_property(DOCKER_ARCHIVE tech.flecs.service-mesh_docker DOCKER_ARCHIVE)
get_filename_component(DOCKER_ARCHIVE ${DOCKER_ARCHIVE} NAME)
string(REPLACE _${ARCH} "" DOCKER_ARCHIVE ${DOCKER_ARCHIVE})
install(
    FILES $<TARGET_PROPERTY:tech.flecs.service-mesh_docker,DOCKER_ARCHIVE>
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/assets
    RENAME ${DOCKER_ARCHIVE}
)

# install App manifests
install(
    FILES $<TARGET_PROPERTY:tech.flecs.mqtt-bridge_app,MANIFEST>
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/assets
)

install(
    FILES $<TARGET_PROPERTY:tech.flecs.service-mesh_app,MANIFEST>
    DESTINATION ${PKGBUILD_DIR}/debian/opt/flecs/assets
)
