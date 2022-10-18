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
set(VERSION 1.5.0-porpoise)
add_definitions(-DFLECS_VERSION="${VERSION}")

set(CMAKE_MODULE_PATH
    ${CMAKE_MODULE_PATH}
    ${CMAKE_SOURCE_DIR}/build/cmake
)

if(FLECS_BUILD_TESTS)
    include(CTest)
    add_compile_options(-fprofile-arcs -ftest-coverage)
    add_link_options(-lgcov --coverage)
    add_definitions(-DFLECS_UNIT_TEST)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED true)
add_compile_options(-Wall -Wextra -Werror -Wfatal-errors)
link_libraries(dl pthread stdc++fs)

execute_process(
    COMMAND ccache -s
    OUTPUT_QUIET
    ERROR_QUIET
    RESULT_VARIABLE CCACHE_FOUND
)

if(CCACHE_FOUND EQUAL 0)
    set(CMAKE_CXX_COMPILER_LAUNCHER ccache)
endif()

set(BOOST_ROOT /opt/boost_1_79_0)
find_package(Boost
    1.78.0
    REQUIRED
)

# hide all symbols by default
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
add_link_options(-Wl,--exclude-libs,ALL)

# add install directory to rpath
set(CMAKE_INSTALL_RPATH "/opt/flecs/lib")

# provide macro to export symbols
add_definitions("-DFLECS_EXPORT=__attribute__\(\(visibility\(\"default\"\)\)\)")

# set MACHINE and ARCH variables
execute_process(
    COMMAND ${CMAKE_CXX_COMPILER} -dumpmachine
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE MACHINE
)

if("${MACHINE}" MATCHES ".*x86_64.*")
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

add_subdirectory(apps)
add_subdirectory(cli)
add_subdirectory(external)
add_subdirectory(flunder)
add_subdirectory(mqtt)
add_subdirectory(util)
add_subdirectory(daemon)

get_property(PACKAGES GLOBAL PROPERTY PACKAGES)

add_custom_target(
    packages
    DEPENDS ${PACKAGES}
    COMMAND echo ${PACKAGES}
)

get_property(DOCKER_IMAGES GLOBAL PROPERTY DOCKER_IMAGES)
add_custom_target(
    docker
    DEPENDS ${DOCKER_IMAGES}
    COMMAND echo ${DOCKER_IMAGES}
)

add_custom_target(
    coverage
    DEPENDS test
    COMMAND mkdir -p coverage
    COMMAND gcovr -r ${CMAKE_SOURCE_DIR} -e ${CMAKE_SOURCE_DIR}/external/ -e ${CMAKE_BINARY_DIR} --html-details coverage/coverage.html
)