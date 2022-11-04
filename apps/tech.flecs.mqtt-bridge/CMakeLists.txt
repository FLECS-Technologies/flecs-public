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

project(FLECS.apps.mqtt-bridge)

add_executable(FLECS.apps.mqtt-bridge
    src/mqtt_bridge.cpp
    mqtt_bridge.h
)
set_target_properties(FLECS.apps.mqtt-bridge PROPERTIES OUTPUT_NAME mqtt_bridge)

# own includes
target_include_directories(FLECS.apps.mqtt-bridge PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(FLECS.apps.mqtt-bridge PRIVATE
    FLECS.external.mosquitto
    FLECS.flunder.static
    FLECS.util.signal_handler
)

# configure Docker image
set(DOCKER_IMAGE "tech.flecs.mqtt-bridge")
set(APP_VERSION ${VERSION})

set(DOCKER_TAG ${APP_VERSION})

install(
    TARGETS FLECS.apps.mqtt-bridge
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/docker
)

install(
    FILES $<TARGET_FILE:FLECS.external.zenoh-c>
    DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/docker
)

include(flecs-docker)
