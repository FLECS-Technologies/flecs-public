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

project(daemon)

add_subdirectory(api)
add_subdirectory(common)
add_subdirectory(modules)

# configure package
set(PACKAGE flecs)
set(PACKAGE_VERSION ${CORE_VERSION})
set(PACKAGE_DESC FLECS)
set(PACKAGE_DEPENDS docker-ce libudev libusb-1.0-0)

add_custom_target(
    flecs_deb-pkg-copy
    COMMAND mkdir -p ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/
    COMMAND cp -f ${CMAKE_SOURCE_DIR}/apps/tech.flecs.service-mesh/manifest.json ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/tech.flecs.service-mesh_${CORE_VERSION}.json
    COMMAND cp -f ${CMAKE_BINARY_DIR}/apps/tech.flecs.service-mesh/tech.flecs.service-mesh_${CORE_VERSION}_${ARCH}.tar ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/tech.flecs.service-mesh_${CORE_VERSION}.tar
    COMMAND cp -f ${CMAKE_SOURCE_DIR}/apps/tech.flecs.mqtt-bridge/manifest.json ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/tech.flecs.mqtt-bridge_${CORE_VERSION}.json
    COMMAND cp -f ${CMAKE_BINARY_DIR}/apps/tech.flecs.mqtt-bridge/tech.flecs.mqtt-bridge_${CORE_VERSION}_${ARCH}.tar ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/tech.flecs.mqtt-bridge_${CORE_VERSION}.tar
)

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
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)

# install shared objects for packaging
install(
    TARGETS daemon.lib mqtt.shared LIBRARY
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

install(
    TARGETS daemon.lib flunder.shared LIBRARY
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

install(
    FILES $<TARGET_FILE:external.zenoh-c>
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

# install FLECS cli for packaging
install(
    TARGETS cli RUNTIME
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)

# install hosts script for packaging
install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/scripts/flecs-update-hosts.sh
    DESTINATION ${PKGBUILD_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)
