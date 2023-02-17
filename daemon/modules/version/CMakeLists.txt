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

flecs_add_module(
    MODULE_NAME version
)

execute_process(
    COMMAND stat -c "%u" ${CMAKE_SOURCE_DIR}/.git
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE FLECS_GIT_UID
    ERROR_VARIABLE FLECS_GIT_UID_ERROR
)
if ("${FLECS_GIT_UID}" STREQUAL "")
    message(FATAL_ERROR ${FLECS_GIT_UID_ERROR})
endif()

execute_process(
    COMMAND sudo -u \#${FLECS_GIT_UID} git -C ${CMAKE_SOURCE_DIR} rev-parse --short HEAD
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE FLECS_GIT_SHA
    ERROR_VARIABLE FLECS_GIT_SHA_ERROR
)
if (NOT FLECS_GIT_SHA)
    message(FATAL_ERROR ${FLECS_GIT_SHA_ERROR})
endif()

target_compile_definitions(FLECS.daemon.modules.version PUBLIC
    -DFLECS_GIT_SHA="${FLECS_GIT_SHA}"
)

add_subdirectory(test)
