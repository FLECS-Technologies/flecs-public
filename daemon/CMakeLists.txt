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
add_subdirectory(app)
add_subdirectory(db)
add_subdirectory(instance)
add_subdirectory(lib)
add_subdirectory(modules)

# configure package
set(PACKAGE flecs)
set(PACKAGE_DESC FLECS)

include(package)

# build executable flecsd
set(SRC_BINARY
    src/daemon.cpp
    src/signal_handler.cpp
)

set(HEADER_BINARY
    daemon.h
    signal_handler.h
)

add_executable(FLECS.daemon ${SRC_BINARY} ${HEADER_BINARY})
set_target_properties(FLECS.daemon PROPERTIES OUTPUT_NAME flecsd)

# own includes
target_include_directories(FLECS.daemon PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# FLECS link libraries
target_link_libraries(FLECS.daemon PRIVATE
    FLECS.daemon.api
    FLECS.daemon.api.endpoints
    FLECS.daemon.modules.app_manager
    FLECS.daemon.modules.factory
    FLECS.daemon.modules.help
    FLECS.daemon.modules.rpc
    FLECS.daemon.modules.usage
    FLECS.daemon.modules.version
    FLECS.util.process
    FLECS.util.curl_easy_ext
    FLECS.util.sqlite3_ext
    FLECS.external.yaml-cpp
)

# install executable flecsd for packaging
install(
    TARGETS FLECS.daemon RUNTIME
    DESTINATION ${PACKAGE_DIR}/debian/usr/bin
)
install(
    TARGETS FLECS.daemon RUNTIME
    DESTINATION ${PACKAGE_DIR}/opkg/usr/bin
)

# install shared object for packaging
install(
    TARGETS FLECS.daemon.lib LIBRARY
    DESTINATION ${PACKAGE_DIR}/debian/usr/lib
)
install(
    TARGETS FLECS.daemon.lib LIBRARY
    DESTINATION ${PACKAGE_DIR}/opkg/usr/lib
)

# install FLECS cli for packaging
install(
    TARGETS FLECS.cli RUNTIME
    DESTINATION ${PACKAGE_DIR}/debian/usr/bin
)
install(
    TARGETS FLECS.cli RUNTIME
    DESTINATION ${PACKAGE_DIR}/opkg/usr/bin
)
