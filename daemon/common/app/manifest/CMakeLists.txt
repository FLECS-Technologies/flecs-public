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

set(PROJECT_NAME daemon.common.app.manifest)

project(${PROJECT_NAME})

add_subdirectory(conffile)
add_subdirectory(env_var)
add_subdirectory(port_range)
add_subdirectory(startup_option)
add_subdirectory(volume)

add_library(
    ${PROJECT_NAME} STATIC
    src/manifest.cpp
    manifest.h
)

target_link_libraries(
    ${PROJECT_NAME} PUBLIC
    daemon.common.network
    daemon.common.app.manifest.conffile
    daemon.common.app.manifest.env_var
    daemon.common.app.manifest.port_range
    daemon.common.app.manifest.startup_option
    daemon.common.app.manifest.volume
)

target_link_libraries(
    ${PROJECT_NAME} PUBLIC
    yaml-cpp::yaml-cpp
)

add_subdirectory(test)
