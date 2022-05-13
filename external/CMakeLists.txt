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

project(FLECS.external)

include(FetchContent)

# define source directories
set (CMAKE_ARGS    -B build/${MACHINE} -DCMAKE_INSTALL_PREFIX=out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_BUILD_TYPE=Release)
set (GTEST_DIR     ${CMAKE_CURRENT_SOURCE_DIR}/gtest-1.11.0)
set (LIBSODIUM_DIR ${CMAKE_CURRENT_SOURCE_DIR}/libsodium-1.0.18)
set (LLHTTP_DIR    ${CMAKE_CURRENT_SOURCE_DIR}/llhttp-v6.0.6)
set (MOSQUITTO_DIR ${CMAKE_CURRENT_SOURCE_DIR}/mosquitto-2.0.14)
set (OPENSSL_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/openssl-1.1.1m)
set (PYBIND11_DIR  ${CMAKE_CURRENT_SOURCE_DIR}/pybind11-2.9.1)
set (SQLITE3_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/sqlite-3.38.2)
set (YAMLCPP_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/yaml-cpp-0.7.0)
set (ZENOH_C_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/zenoh-c-0.5.0-beta.9)

# cpr
FetchContent_Declare(
    cpr
    GIT_REPOSITORY https://github.com/libcpr/cpr/
    GIT_TAG        db351ff
)

set_property(
    DIRECTORY ${cpr_SOURCE_DIR}
    APPEND
    PROPERTY COMPILE_OPTIONS -Wno-error -fPIC
)

set (BUILD_SHARED_LIBS NO CACHE BOOL "" FORCE)
set (OPENSSL_ROOT_DIR ${OPENSSL_DIR}/out/${MACHINE} CACHE PATH "" FORCE)
set (OPENSSL_INCLUDE_DIR ${OPENSSL_DIR}/out/${MACHINE}/include CACHE PATH "" FORCE)

FetchContent_MakeAvailable(cpr)

add_library(FLECS.external.cpr INTERFACE)
target_link_libraries(FLECS.external.cpr INTERFACE cpr::cpr)

# nlohmann/json
FetchContent_Declare(
    external.nlohmann.json
    GIT_REPOSITORY https://github.com/nlohmann/json/
    GIT_TAG        4f8fba1 # v3.10.5
)

FetchContent_MakeAvailable(external.nlohmann.json)

add_library(FLECS.external.json INTERFACE)
target_include_directories(FLECS.external.json INTERFACE ${external.nlohmann.json_SOURCE_DIR}/include)

# custom build rules
# libsodium
add_custom_target(
    FLECS.external.libsodium.build
    WORKING_DIRECTORY ${LIBSODIUM_DIR}
    COMMAND ./configure --prefix=${LIBSODIUM_DIR}/out/${MACHINE} --host=${CROSS_COMPILE}
    COMMAND ${CMAKE_MAKE_PROGRAM} install
    COMMAND ${CMAKE_MAKE_PROGRAM} clean
)

# llhttp
add_custom_target(
    FLECS.external.llhttp.build
    WORKING_DIRECTORY ${LLHTTP_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${MACHINE}
    COMMAND ${CMAKE_COMMAND} --install build/${MACHINE}
)

# mosquitto
add_custom_target(
    FLECS.external.mosquitto.build
    WORKING_DIRECTORY ${MOSQUITTO_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DDOCUMENTATION=NO -DWITH_STATIC_LIBRARIES=YES -DWITH_PIC=YES -DWITH_APPS=NO -DWITH_BROKER=NO -DWITH_CLIENTS=NO -DWITH_PLUGINS=NO -DOPENSSL_ROOT_DIR=${OPENSSL_DIR}/out/${MACHINE}
    COMMAND ${CMAKE_COMMAND} --build build/${MACHINE}
    COMMAND ${CMAKE_COMMAND} --install build/${MACHINE}
)

# OpenSSL
add_custom_target(
    FLECS.external.openssl.build
    WORKING_DIRECTORY ${OPENSSL_DIR}
    COMMAND flecs-configure/Configure.${MACHINE}.sh ${OPENSSL_DIR} --cross-compile-prefix= --prefix=${OPENSSL_DIR}/out/${MACHINE} no-shared
	COMMAND ${CMAKE_MAKE_PROGRAM} CROSS_COMPILE= all
	COMMAND ${CMAKE_MAKE_PROGRAM} CROSS_COMPILE= install_sw
	COMMAND ${CMAKE_MAKE_PROGRAM} clean
)

# SQLite3
add_custom_target(
    FLECS.external.sqlite3.build
    WORKING_DIRECTORY ${SQLITE3_DIR}
    COMMAND autoreconf
    COMMAND ./configure --prefix=${SQLITE3_DIR}/out/${MACHINE} --host=${CROSS_COMPILE}
    COMMAND make install
    COMMAND make clean
)

# yaml-cpp
add_custom_target(
    FLECS.external.yaml-cpp.build
    WORKING_DIRECTORY ${YAMLCPP_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DYAML_CPP_BUILD_TESTS=NO
    COMMAND ${CMAKE_COMMAND} --build build/${MACHINE}
    COMMAND ${CMAKE_COMMAND} --install build/${MACHINE}
)

# Zenoh-C
# Use prebuilt

# imported libraries
# gtest
add_library(FLECS.external.gmock_main STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gmock_main PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/lib/libgmock_main.a)
set_property(TARGET FLECS.external.gmock_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR})

add_library(FLECS.external.gmock STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gmock PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/lib/libgmock.a)
set_property(TARGET FLECS.external.gmock PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR})

add_library(FLECS.external.gtest_main STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gtest_main PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/lib/libgtest_main.a)
set_property(TARGET FLECS.external.gtest_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR})

add_library(FLECS.external.gtest STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gtest PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/lib/libgtest.a)
set_property(TARGET FLECS.external.gtest PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR})

# libsodium
add_library(FLECS.external.libsodium STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.libsodium PROPERTY IMPORTED_LOCATION ${LIBSODIUM_DIR}/out/${MACHINE}/lib/libsodium.a)
set_property(TARGET FLECS.external.libsodium PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${LIBSODIUM_DIR}/out/${MACHINE}/include)

# llhttp
add_library(FLECS.external.llhttp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.llhttp PROPERTY IMPORTED_LOCATION ${LLHTTP_DIR}/out/${MACHINE}/lib/libllhttp.a)
set_property(TARGET FLECS.external.llhttp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${LLHTTP_DIR}/include)

# mosquitto
add_library(FLECS.external.mosquitto STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.mosquitto PROPERTY IMPORTED_LOCATION ${MOSQUITTO_DIR}/out/${MACHINE}/lib/libmosquitto_static.a)
set_property(TARGET FLECS.external.mosquitto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${MOSQUITTO_DIR}/include)
target_link_libraries(FLECS.external.mosquitto INTERFACE
    FLECS.external.ssl
    FLECS.external.crypto
)

# OpenSSL
add_library(FLECS.external.crypto STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.crypto PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${MACHINE}/lib/libcrypto.a)
set_property(TARGET FLECS.external.crypto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${MACHINE}/include)

add_library(FLECS.external.ssl STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.ssl PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${MACHINE}/lib/libssl.a)
set_property(TARGET FLECS.external.ssl PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${MACHINE}/include)

# pybind11
add_library(FLECS.external.pybind11 INTERFACE)
target_include_directories(FLECS.external.pybind11 INTERFACE
    ${PYBIND11_DIR}/include
)

# SQLite3
add_library(FLECS.external.sqlite3 STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.sqlite3 PROPERTY IMPORTED_LOCATION ${SQLITE3_DIR}/out/${MACHINE}/lib/libsqlite3.a)
set_property(TARGET FLECS.external.sqlite3 PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${SQLITE3_DIR})

# yaml-cpp
add_library(FLECS.external.yaml-cpp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY IMPORTED_LOCATION ${YAMLCPP_DIR}/out/${MACHINE}/lib/libyaml-cpp.a)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${YAMLCPP_DIR}/include)

# Zenoh-C
add_library(FLECS.external.zenoh-c SHARED IMPORTED GLOBAL)
set_property(TARGET FLECS.external.zenoh-c PROPERTY IMPORTED_LOCATION ${ZENOH_C_DIR}/out/${MACHINE}/lib/libzenohc.so)
set_property(TARGET FLECS.external.zenoh-c PROPERTY IMPORTED_NO_SONAME TRUE)
set_property(TARGET FLECS.external.zenoh-c PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${ZENOH_C_DIR}/include)
