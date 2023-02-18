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

project(FLECS.flunder)

set(SRC_LIB
    src/flunder_client.cpp
    src/flunder_variable.cpp
    src/impl/flunder_client_impl.cpp
)

set(HEADER_LIB
    flunder_client.h
    flunder_variable.h
    impl/flunder_client_impl.h
)

add_library(FLECS.flunder OBJECT ${SRC_LIB} ${HEADER_LIB})

set_property(TARGET FLECS.flunder PROPERTY POSITION_INDEPENDENT_CODE 1)

add_library(FLECS.flunder.shared SHARED $<TARGET_OBJECTS:FLECS.flunder>)
set_target_properties(FLECS.flunder.shared PROPERTIES OUTPUT_NAME FLECS.flunder)

add_library(FLECS.flunder.static STATIC $<TARGET_OBJECTS:FLECS.flunder>)
set_target_properties(FLECS.flunder.static PROPERTIES OUTPUT_NAME FLECS.flunder)

target_include_directories(FLECS.flunder PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.flunder PRIVATE
    external.cpr
    FLECS.external.json
    FLECS.external.zenoh-c
)

target_link_libraries(FLECS.flunder.static INTERFACE
    external.cpr
    FLECS.external.zenoh-c
)

target_link_libraries(FLECS.flunder.shared PRIVATE
    -Wl,--whole-archive
    external.cpr
    FLECS.external.zenoh-c
    -Wl,--no-whole-archive
)

add_subdirectory(examples)
add_subdirectory(test)

# add_subdirectory(pybind)
