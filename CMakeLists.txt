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

cmake_minimum_required(VERSION 3.13)

project(FLECS)

# set version
set(VERSION 1.0.0-beta.1)
add_definitions(-DFLECS_VERSION="${VERSION}")

set(CMAKE_MODULE_PATH
    ${CMAKE_MODULE_PATH}
    ${CMAKE_SOURCE_DIR}/build/cmake
)

if (FLECS_BUILD_TESTS)
include (CTest)
add_compile_options(-fprofile-arcs -ftest-coverage)
add_link_options(-lgcov --coverage)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED true)
add_compile_options(-Wall -Wextra -Werror -Wfatal-errors)

# hide all symbols by default
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
add_link_options(-Wl,--exclude-libs,ALL)

# provide macro to export symbols
add_definitions("-DFLECS_EXPORT=__attribute__\(\(visibility\(\"default\"\)\)\)")

# set MACHINE and ARCH variables
execute_process(
    COMMAND ${CMAKE_CXX_COMPILER} -dumpmachine
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE MACHINE
)

if("${MACHINE}" STREQUAL x86_64-linux-gnu)
    set(ARCH amd64)
    set(DOCKER_ARCH linux/amd64)
elseif("${MACHINE}" STREQUAL arm-linux-gnueabihf)
	set(ARCH armhf)
    set(DOCKER_ARCH linux/arm/v7)
elseif("${MACHINE}" STREQUAL aarch64-linux-gnu)
	set(ARCH arm64)
    set(DOCKER_ARCH linux/arm64)
else()
    message(FATAL_ERROR "Could not determine architecture for ${MACHINE}")
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

add_subdirectory(cli)
add_subdirectory(daemon)
add_subdirectory(develop)
add_subdirectory(external)
add_subdirectory(flunder)
add_subdirectory(mqtt)
add_subdirectory(updater)
add_subdirectory(util)

add_custom_target(
    packages
    DEPENDS ${PACKAGES}
    COMMAND echo ${PACKAGES}
)

add_custom_target(
    docker
    DEPENDS ${DOCKER_IMAGES}
    COMMAND echo ${DOCKER_IMAGES}
)
