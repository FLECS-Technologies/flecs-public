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

project(FLECS.backend)

# configure package
set(PACKAGE flecs-backend)
set(PACKAGE_DESC "FLECS HTTP backend")

include(package)

# build executable flecsd
set(SRC_BINARY
    src/main.cpp
    src/http_request_handler.cpp
)

set(HEADER_BINARY
    http_request_handler.h
)

add_executable(flecs-backend ${SRC_BINARY} ${HEADER_BINARY})

# own includes
target_include_directories(flecs-backend PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# FLECS link libraries
target_link_libraries(flecs-backend PRIVATE
    FLECS.util.curl_easy_ext
    FLECS.util.llhttp_ext
    FLECS.util.process
    FLECS.external.jsoncpp
    pthread
)

# install executable flecsd for packaging
install(
    TARGETS flecs-backend RUNTIME
    DESTINATION ${PACKAGE_DIR}/debian/usr/bin
)
install(
    TARGETS flecs-backend RUNTIME
    DESTINATION ${PACKAGE_DIR}/opkg/usr/bin
)
