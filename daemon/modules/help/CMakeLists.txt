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

project(FLECS.daemon.modules.help)

add_library(
    FLECS.daemon.modules.help OBJECT
    src/help.cpp
    help.h
)

target_include_directories(
    FLECS.daemon.modules.help INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../
)

target_include_directories(
    FLECS.daemon.modules.help PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.daemon.modules.help PRIVATE
    FLECS.daemon.modules.module_base
)
