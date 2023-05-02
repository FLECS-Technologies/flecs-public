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

project(mqtt)

set(SRC_LIB
    src/mqtt_client.cpp
    src/impl/mqtt_client_impl.cpp
)

set(INCLUDE_LIB
    mqtt_client.h
    impl/mqtt_client_impl.h
)

add_library(mqtt OBJECT ${SRC_LIB} ${HEADER_LIB})

set_property(TARGET mqtt PROPERTY POSITION_INDEPENDENT_CODE 1)

add_library(mqtt.shared SHARED $<TARGET_OBJECTS:mqtt>)
set_target_properties(mqtt.shared PROPERTIES OUTPUT_NAME FLECS.mqtt)

add_library(mqtt.static STATIC $<TARGET_OBJECTS:mqtt>)
set_target_properties(mqtt.static PROPERTIES OUTPUT_NAME FLECS.mqtt)

target_include_directories(mqtt PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(mqtt PRIVATE
    external.mosquitto
)

target_link_libraries(mqtt.static INTERFACE
    external.mosquitto
)

target_link_libraries(mqtt.shared PRIVATE
    -Wl,--whole-archive
    external.mosquitto
    -Wl,--no-whole-archive
)
