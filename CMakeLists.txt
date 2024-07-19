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

cmake_minimum_required(VERSION 3.21)

include(${CMAKE_SOURCE_DIR}/flecs-build/cmake/flecs-build.cmake)

project(
    FLECS
    VERSION 3.4.0
    HOMEPAGE_URL https://flecs.tech
    LANGUAGES CXX
)
set(CODENAME hedgehog)

# set version
set(FLECS_VERSION_FULL "${PROJECT_VERSION}-${CODENAME}")
if (NOT "${CMAKE_BUILD_TYPE}" STREQUAL "Release")
    set(FLECS_VERSION_FULL "${FLECS_VERSION_FULL}-dev")
endif()

set(API_VERSION 2.0.0)
add_definitions(-DFLECS_VERSION="${FLECS_VERSION_FULL}")
add_definitions(-DFLECS_API_VERSION="${API_VERSION}")

# workaround for gcc-12 bug
add_compile_options(-Wno-error=restrict)

if(FLECS_BUILD_TESTS)
    include(CTest)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

set(FLECS_NEED_CPR TRUE)
set(FLECS_NEED_JSON TRUE)
set(FLECS_NEED_LIBARCHIVE TRUE)
set(FLECS_NEED_LIBUSB TRUE)
set(FLECS_NEED_YAMLCPP TRUE)
set(FLECS_NEED_ZENOHC TRUE)
add_subdirectory(flecs-external)
add_subdirectory(flunder)
add_subdirectory(flecs)

add_custom_target(version
    COMMAND echo "${FLECS_VERSION_FULL}"
)

add_subdirectory(docker)
add_subdirectory(pkg)

include(flecs-rules)
