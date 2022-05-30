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

set(PROJECT_NAME FLECS.daemon.common.app.manifest)

project(${PROJECT_NAME})

add_subdirectory(conffile)
add_subdirectory(env_var)
add_subdirectory(port_range)
add_subdirectory(volume)

add_library(
    ${PROJECT_NAME} STATIC
    src/manifest.cpp
    manifest.h
)

target_include_directories(
    ${PROJECT_NAME} INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../
)

target_include_directories(
    ${PROJECT_NAME} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(
    ${PROJECT_NAME} PRIVATE
    FLECS.daemon.common.app.manifest.conffile
    FLECS.daemon.common.app.manifest.env_var
    FLECS.daemon.common.app.manifest.port_range
    FLECS.daemon.common.app.manifest.volume
)

target_link_libraries(
    ${PROJECT_NAME} PUBLIC
    FLECS.external.json
    FLECS.external.yaml-cpp
)

add_subdirectory(test)
