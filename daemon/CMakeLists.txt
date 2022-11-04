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

project(FLECS.daemon)

add_subdirectory(api)
add_subdirectory(common)
add_subdirectory(modules)

# configure package
set(PACKAGE flecs)
set(PACKAGE_DESC FLECS)
set(PACKAGE_DEPENDS docker-ce libudev libusb-1.0-0)

add_custom_target(
    flecs_deb-pkg-copy
    COMMAND mkdir -p ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/
    COMMAND cp -f ${CMAKE_BINARY_DIR}/apps/tech.flecs.service-mesh/tech.flecs.service-mesh_${VERSION}_${ARCH}.tar.gz ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/
    COMMAND cp -f ${CMAKE_BINARY_DIR}/apps/tech.flecs.mqtt-bridge/tech.flecs.mqtt-bridge_${VERSION}_${ARCH}.tar.gz ${CMAKE_CURRENT_BINARY_DIR}/pkg/debian/opt/flecs/assets/
)

include(flecs-package)

# build executable flecsd
add_executable(FLECS.daemon
    src/main.cpp
)
set_target_properties(FLECS.daemon PROPERTIES OUTPUT_NAME flecsd)

# own includes
target_include_directories(FLECS.daemon PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# FLECS link libraries
target_link_libraries(FLECS.daemon PRIVATE
    FLECS.daemon.api
    FLECS.daemon.common
    FLECS.daemon.modules.app_manager
    FLECS.daemon.modules.data_layer
    FLECS.daemon.modules.factory
    FLECS.daemon.modules.help
    FLECS.daemon.modules.marketplace
    FLECS.daemon.modules.module_base
    FLECS.daemon.modules.rpc
    FLECS.daemon.modules.system
    FLECS.daemon.modules.usage
    FLECS.daemon.modules.version
    FLECS.util.process
    FLECS.util.signal_handler
    FLECS.util.sqlite3_ext
    FLECS.external.yaml-cpp
)

# install executable flecsd for packaging
install(
    TARGETS FLECS.daemon RUNTIME
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)

# install shared objects for packaging
install(
    TARGETS FLECS.daemon.lib FLECS.mqtt.shared LIBRARY
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

install(
    TARGETS FLECS.daemon.lib FLECS.flunder.shared LIBRARY
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

install(
    FILES $<TARGET_FILE:FLECS.external.zenoh-c>
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/lib
)

# install FLECS cli for packaging
install(
    TARGETS FLECS.cli RUNTIME
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)

# install hosts script for packaging
install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/scripts/flecs-update-hosts.sh
    DESTINATION ${PACKAGE_DIR}/debian/${FLECS_INSTALL_DIR}/bin
)
