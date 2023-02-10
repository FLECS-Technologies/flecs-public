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

project(FLECS.mqtt.pybind)

execute_process(
    COMMAND python3 -m pybind11 --includes
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE PYBIND_INCLUDES
)

execute_process(
    COMMAND python3-config --extension-suffix
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE PYBIND_SUFFIX
)

separate_arguments(PYBIND_INCLUDES UNIX_COMMAND "${PYBIND_INCLUDES}")

add_library(FLECS.mqtt_client.pybind SHARED mqtt_client_pybind.cpp)
set_target_properties(FLECS.mqtt_client.pybind PROPERTIES LIBRARY_OUTPUT_NAME mqtt_client${PYBIND_SUFFIX})
set_target_properties(FLECS.mqtt_client.pybind PROPERTIES PREFIX "")
set_target_properties(FLECS.mqtt_client.pybind PROPERTIES SUFFIX "")

add_library(FLECS.mqtt_message.pybind SHARED mqtt_message_pybind.cpp)
set_target_properties(FLECS.mqtt_message.pybind PROPERTIES LIBRARY_OUTPUT_NAME mqtt_message${PYBIND_SUFFIX})
set_target_properties(FLECS.mqtt_message.pybind PROPERTIES PREFIX "")
set_target_properties(FLECS.mqtt_message.pybind PROPERTIES SUFFIX "")

foreach(path ${PYBIND_INCLUDES})
    target_compile_options(FLECS.mqtt_client.pybind PRIVATE
        ${path}
    )
    target_compile_options(FLECS.mqtt_message.pybind PRIVATE
        ${path}
    )
endforeach()

target_link_libraries(FLECS.mqtt_client.pybind PRIVATE
    external.pybind11
    FLECS.mqtt.shared
)

target_link_libraries(FLECS.mqtt_message.pybind PRIVATE
    external.pybind11
)
