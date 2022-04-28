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

project(FLECS.daemon.app)

add_subdirectory(conffile)
add_subdirectory(env_var)
add_subdirectory(port_range)

add_library(
    FLECS.daemon.app STATIC
    src/app.cpp
    app.h app_status.h
)

target_include_directories(
    FLECS.daemon.app INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../
)

target_include_directories(
    FLECS.daemon.app PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.daemon.app PUBLIC
    FLECS.external.yaml-cpp
)

target_link_libraries(FLECS.daemon.app PRIVATE
    FLECS.daemon.app.conffile
    FLECS.daemon.app.env_var
    FLECS.daemon.app.port_range
)

add_subdirectory(test)
