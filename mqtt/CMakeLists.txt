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

project(FLECS.mqtt)

set(SRC_LIB
    src/mqtt_client.cpp
    src/private/mqtt_client_private.cpp
)

set(INCLUDE_LIB
    mqtt_client.h
    private/mqtt_client_private.h
)

add_library(FLECS.mqtt OBJECT ${SRC_LIB} ${HEADER_LIB})

set_property(TARGET FLECS.mqtt PROPERTY POSITION_INDEPENDENT_CODE 1)

add_library(FLECS.mqtt.shared SHARED $<TARGET_OBJECTS:FLECS.mqtt>)
set_target_properties(FLECS.mqtt.shared PROPERTIES OUTPUT_NAME FLECS.mqtt)

add_library(FLECS.mqtt.static STATIC $<TARGET_OBJECTS:FLECS.mqtt>)
set_target_properties(FLECS.mqtt.static PROPERTIES OUTPUT_NAME FLECS.mqtt)

target_include_directories(FLECS.mqtt PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.mqtt PRIVATE
    FLECS.external.mosquitto
)
