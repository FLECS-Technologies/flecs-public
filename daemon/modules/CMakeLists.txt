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

project(daemon.modules)

macro(flecs_add_module)
    set(options "")
    set(oneValueArgs MODULE_NAME)
    set(multiValueArgs ADDITIONAL_SOURCES ADDITIONAL_HEADERS LIBS_PRIVATE LIBS_PUBLIC)
    cmake_parse_arguments(
        mod "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN}
    )

    project(daemon.modules.${mod_MODULE_NAME})

    add_library(
        daemon.modules.${mod_MODULE_NAME} OBJECT
        src/${mod_MODULE_NAME}.cpp ${mod_ADDITIONAL_SOURCES}
        ${mod_MODULE_NAME}.h ${mod_ADDITIONAL_HEADERS}
    )

    target_include_directories(
        daemon.modules.${mod_MODULE_NAME} INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../
    )

    target_include_directories(
        daemon.modules.${mod_MODULE_NAME} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
    )

    target_link_libraries(daemon.modules.${mod_MODULE_NAME} PUBLIC
        daemon.api
        daemon.modules.factory
        daemon.modules.module_base
        ${mod_LIBS_PUBLIC}
    )

    target_link_libraries(daemon.modules.${mod_MODULE_NAME} PRIVATE
        ${mod_LIBS_PRIVATE}
    )
endmacro()

add_subdirectory(apps)
add_subdirectory(console)
add_subdirectory(data_layer)
add_subdirectory(device)
add_subdirectory(factory)
add_subdirectory(flecsport)
add_subdirectory(instances)
add_subdirectory(jobs)
add_subdirectory(manifests)
add_subdirectory(module_base)
add_subdirectory(system)
add_subdirectory(version)
