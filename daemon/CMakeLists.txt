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

# configure package
set(PACKAGE flecs)
set(PACKAGE_DESC FLECS)

include(package)

# build executable flecsd
set(SRC_BINARY
    src/daemon.cpp
    src/main.cpp
    src/api/api.cpp
    src/app/app.cpp
    src/db/app_db.cpp
    src/modules/private/app_manager_private.cpp
    src/modules/app_manager.cpp
    src/modules/help.cpp
    src/modules/module.cpp
    src/modules/rpc.cpp
    src/modules/usage.cpp
    src/signal_handler/signal_handler.cpp
)

set(HEADER_BINARY
    api/api.h
    app/app.h
    app/app_status.h
    db/app_db.h
    daemon.h
    instance/instance_status.h
    modules/private/app_manager_private.h
    modules/app_manager.h
    modules/errors.h
    modules/factory.h
    modules/help.h
    modules/module.h
    modules/rpc.h
    modules/usage.h
    signal_handler/signal_handler.h
)

add_executable(FLECS.daemon ${SRC_BINARY} ${HEADER_BINARY})
set_target_properties(FLECS.daemon PROPERTIES OUTPUT_NAME flecsd)

# own includes
target_include_directories(FLECS.daemon PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# FLECS link libraries
target_link_libraries(FLECS.daemon PRIVATE
    FLECS.util.process
    FLECS.util.curl_easy_ext
    FLECS.util.sqlite3_ext
    FLECS.external.jsoncpp
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

# build shared object
set(SRC_LIB
    src/lib/libflecs.cpp
    src/lib/private/libflecs_private.cpp
)

set(HEADER_LIB
    lib/libflecs.h
    lib/private/libflecs_private.h
)

add_library(FLECS.libdaemon SHARED ${SRC_LIB} ${HEADER_LIB})

# own includes
target_include_directories(FLECS.libdaemon PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# install shared object for packaging
install(
    TARGETS FLECS.libdaemon LIBRARY
    DESTINATION ${PACKAGE_DIR}/debian/usr/lib
)
install(
    TARGETS FLECS.libdaemon LIBRARY
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
