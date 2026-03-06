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

project(FLECS)

# read version from .cargo/config.toml (single source of truth)
execute_process(
    COMMAND tomlq -r ".env.FLECS_VERSION" ${CMAKE_SOURCE_DIR}/.cargo/config.toml
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE FLECS_VERSION
)
execute_process(
    COMMAND tomlq -r ".env.FLECS_CODENAME" ${CMAKE_SOURCE_DIR}/.cargo/config.toml
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE FLECS_CODENAME
)
if("${FLECS_VERSION}" STREQUAL "" OR "${FLECS_CODENAME}" STREQUAL "")
    message(FATAL_ERROR "Could not determine FLECS version from .cargo/config.toml")
endif()
set(FLECS_VERSION_FULL "${FLECS_VERSION}-${FLECS_CODENAME}")
if(NOT "${CMAKE_BUILD_TYPE}" STREQUAL "Release")
    set(FLECS_VERSION_FULL "${FLECS_VERSION_FULL}-dev")
endif()
message(STATUS "FLECS version: ${FLECS_VERSION_FULL}")

# detect or validate ARCH and set CARGO_TARGET
if(NOT ARCH)
    execute_process(
        COMMAND uname -m
        OUTPUT_STRIP_TRAILING_WHITESPACE
        OUTPUT_VARIABLE MACHINE
    )
    if("${MACHINE}" MATCHES "x86_64")
        set(ARCH amd64)
        set(CARGO_TARGET "x86_64-unknown-linux-gnu")
    elseif("${MACHINE}" MATCHES "aarch64")
        set(ARCH arm64)
        set(CARGO_TARGET "aarch64-unknown-linux-gnu")
    else()
        message(FATAL_ERROR "Could not determine architecture for ${MACHINE}")
    endif()
    message(STATUS "Detected target machine ${MACHINE}")
else()
    if("${ARCH}" STREQUAL "amd64")
        set(CARGO_TARGET "x86_64-unknown-linux-gnu")
    elseif("${ARCH}" STREQUAL "arm64")
        set(CARGO_TARGET "aarch64-unknown-linux-gnu")
    else()
        message(FATAL_ERROR "Invalid target architecture ${ARCH} specified")
    endif()
endif()

message(STATUS "ARCH: ${ARCH}, CARGO_TARGET: ${CARGO_TARGET}")

if("${CMAKE_BUILD_TYPE}" STREQUAL "Release")
    set(CARGO_PROFILE "release")
    set(CARGO_PROFILE_ARG "--release")
else()
    set(CARGO_PROFILE "debug")
    set(CARGO_PROFILE_ARG "")
endif()

set(CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo")
set(FLECSD_BINARY "${CARGO_TARGET_DIR}/${CARGO_TARGET}/${CARGO_PROFILE}/flecsd")

add_custom_target(flecsd_cargo ALL
    COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${CARGO_TARGET_DIR}
        ${CMAKE_SOURCE_DIR}/auditable-cargo build ${CARGO_PROFILE_ARG} --target ${CARGO_TARGET}
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${FLECSD_BINARY} ${CMAKE_BINARY_DIR}/flecsd
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    BYPRODUCTS ${CMAKE_BINARY_DIR}/flecsd
    COMMENT "Building flecsd with cargo"
    USES_TERMINAL
)

add_subdirectory(docker)

get_property(DOCKER_IMAGES GLOBAL PROPERTY DOCKER_IMAGES)
add_custom_target(
    docker
    DEPENDS ${DOCKER_IMAGES}
    COMMAND echo ${DOCKER_IMAGES}
)
