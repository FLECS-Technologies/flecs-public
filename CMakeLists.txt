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
    VERSION 5.0.0
    HOMEPAGE_URL https://flecs.tech
    LANGUAGES CXX
)
set(CODENAME red-deer)

# set version
set(FLECS_VERSION_FULL "${PROJECT_VERSION}-${CODENAME}")
if (NOT "${CMAKE_BUILD_TYPE}" STREQUAL "Release")
    set(FLECS_VERSION_FULL "${FLECS_VERSION_FULL}-dev")
endif()

set(API_VERSION 2.0.0)
add_definitions(-DFLECS_VERSION="${FLECS_VERSION_FULL}")
add_definitions(-DFLECS_API_VERSION="${API_VERSION}")

set(Rust_CARGO ${CMAKE_SOURCE_DIR}/auditable-cargo)

include(FetchContent)

FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG b1fab721655c5c4b1b08a083d3cd29f163af75d0
)

if ("${ARCH}" STREQUAL "amd64")
    set(Rust_CARGO_TARGET "x86_64-unknown-linux-gnu")
elseif("${ARCH}" STREQUAL "arm64")
    set(Rust_CARGO_TARGET "aarch64-unknown-linux-gnu")
endif()

message(STATUS "Rust_CARGO_TARGET: ${Rust_CARGO_TARGET}")

# Set any global configuration variables such as `Rust_TOOLCHAIN` before this line!
FetchContent_MakeAvailable(Corrosion)

# workaround for gcc-12 bug
add_compile_options(-Wno-error=restrict)

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

corrosion_import_crate(MANIFEST_PATH "${CMAKE_SOURCE_DIR}/Cargo.toml")

add_custom_target(version
    COMMAND echo "${FLECS_VERSION_FULL}"
)

add_subdirectory(docker)
add_subdirectory(pkg)

include(flecs-rules)
