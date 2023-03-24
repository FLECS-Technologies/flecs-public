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

cmake_minimum_required(VERSION 3.13)

include(${CMAKE_SOURCE_DIR}/flecs-build/cmake/flecs-build.cmake)

project(FLECS)

# set version
set(CORE_VERSION 2.0.2-dormouse)
set(API_VERSION 2.0.0)
add_definitions(-DFLECS_VERSION="${CORE_VERSION}")
add_definitions(-DFLECS_API_VERSION="${API_VERSION}")

if(FLECS_BUILD_TESTS)
    include(CTest)
    add_compile_options(-fprofile-arcs -ftest-coverage)
    add_link_options(-lgcov --coverage)
    add_definitions(-DFLECS_UNIT_TEST)
    add_definitions(-DFLECS_CONSTEXPR_UNLESS_TESTED=)
    add_definitions(-DFLECS_FINAL_UNLESS_TESTED=)
else()
    add_definitions(-DFLECS_CONSTEXPR_UNLESS_TESTED=constexpr)
    add_definitions(-DFLECS_FINAL_UNLESS_TESTED=final)
endif()

set(BOOST_ROOT /opt/boost)
find_package(Boost
    1.78.0
    REQUIRED
)

# add install directory to rpath
set(CMAKE_INSTALL_RPATH "/opt/flecs/lib")

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

add_subdirectory(apps)
add_subdirectory(cli)
add_subdirectory(external)
add_subdirectory(flunder)
add_subdirectory(mqtt)
add_subdirectory(util)
add_subdirectory(daemon)

include(flecs-rules)

add_custom_command(TARGET packages POST_BUILD
    COMMAND echo -n ${CORE_VERSION} >latest_flecs_${ARCH}
)

add_custom_target(
    coverage
    DEPENDS test
    COMMAND mkdir -p coverage
    COMMAND gcovr -r ${CMAKE_SOURCE_DIR} -e ${CMAKE_SOURCE_DIR}/external/ -e ${CMAKE_BINARY_DIR} --html-details coverage/coverage.html
)
