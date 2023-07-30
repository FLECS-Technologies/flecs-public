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

project(apps.mqtt-bridge)

add_executable(apps.mqtt-bridge
    src/mqtt_bridge.cpp
    mqtt_bridge.h
)
set_target_properties(apps.mqtt-bridge PROPERTIES OUTPUT_NAME mqtt_bridge)

# own includes
target_include_directories(apps.mqtt-bridge PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(apps.mqtt-bridge PRIVATE
    external.mosquitto
    flunder.static
    util.signal_handler
)

# configure FLECS App
set(APP_NAME tech.flecs.mqtt-bridge)
set(APP_VERSION ${CORE_VERSION})

include(flecs-app)

# configure Docker image
set(DOCKER_IMAGE ${APP_NAME})
set(DOCKER_TAG "${APP_VERSION}")

install(
    TARGETS apps.mqtt-bridge
    DESTINATION ${PROJECT_NAME}/docker/fs/usr/bin/
)

install(
    FILES $<TARGET_FILE:external.zenoh-c>
    DESTINATION ${PROJECT_NAME}/docker/fs/usr/lib/
)

include(flecs-docker)
