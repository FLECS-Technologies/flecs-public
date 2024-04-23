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

include(../corrosion_cxx.cmake)
add_library_rust(PATH ../flecs-core-rs NAMESPACE my)

set(CXXBRIDGE_INCLUDE_PATH "${CMAKE_BINARY_DIR}/cargo/build/${Rust_CARGO_TARGET}")
message(STATUS "CXXBRIDGE_INCLUDE_PATH=${CXXBRIDGE_INCLUDE_PATH}")
include_directories(
    ${CXXBRIDGE_INCLUDE_PATH}
)

add_subdirectory(util)
add_subdirectory(api)
add_subdirectory(common)
add_subdirectory(modules)

# build executable flecsd
add_executable(daemon
    src/main.cpp
)
set_target_properties(daemon PROPERTIES OUTPUT_NAME flecsd)

# FLECS link libraries
target_link_libraries(daemon PRIVATE
    api
    common
    modules.apps
    modules.console
    modules.data_layer
    modules.deployments
    modules.device
    modules.factory
    modules.flecsport
    modules.instances
    modules.jobs
    modules.manifests
    modules.module_base
    modules.system
    modules.version
    util.datetime
    util.process
    util.signal_handler
    yaml-cpp::yaml-cpp
)

include(GNUInstallDirs)

install(TARGETS daemon
    RUNTIME
    DESTINATION ${CMAKE_INSTALL_BINDIR}
    COMPONENT native
)
install(FILES ${FLECS_SOURCE_DIR}/pkg/fs/lib/systemd/system/flecs.service
    DESTINATION /lib/systemd/system/
    COMPONENT native-systemd
    EXCLUDE_FROM_ALL
)
install(
    TARGETS lib flunder.shared LIBRARY
    DESTINATION ${CMAKE_INSTALL_LIBDIR}
    COMPONENT native-sysv
    EXCLUDE_FROM_ALL
)
