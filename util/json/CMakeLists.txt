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

project(FLECS.util.json)

add_library(FLECS.util.json OBJECT
    src/json_parser.cpp
    json_parser.h
)

target_include_directories(FLECS.util.json PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.util.json PUBLIC
    FLECS.external.jsoncpp
)
