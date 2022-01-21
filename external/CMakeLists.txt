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

include(ExternalProject)

# define source directories

set (CURL_DIR      ${CMAKE_CURRENT_SOURCE_DIR}/curl-7.80.0)
set (GTEST_DIR     ${CMAKE_CURRENT_SOURCE_DIR}/gtest-1.11.0)
set (JSONCPP_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/jsoncpp-1.9.5)
set (LLHTTP_DIR    ${CMAKE_CURRENT_SOURCE_DIR}/llhttp-2.1.4)
set (MOSQUITTO_DIR ${CMAKE_CURRENT_SOURCE_DIR}/mosquitto-2.0.14)
set (OPENSSL_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/openssl-1.1.1m)
set (SQLITE3_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/sqlite-3.37.0)
set (YAMLCPP_DIR   ${CMAKE_CURRENT_SOURCE_DIR}/yaml-cpp-0.7.0)
set (ZLIB_DIR      ${CMAKE_CURRENT_SOURCE_DIR}/zlib-1.2.11)

# custom build rules

# cURL
add_custom_target(
    FLECS.external.curl.build
    #DEPENDS FLECS.external.openssl.build
    WORKING_DIRECTORY ${CURL_DIR}
    COMMAND ${CMAKE_COMMAND} -B out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DBUILD_SHARED_LIBS=NO -DOPENSSL_ROOT_DIR=${OPENSSL_DIR}/out/${MACHINE} -DOPENSSL_INCLUDE_DIR=${OPENSSL_DIR}/out/${MACHINE}/include
    COMMAND ${CMAKE_COMMAND} --build out/${MACHINE}
)

# JsonCpp
add_custom_target(
    FLECS.external.jsoncpp.build
    WORKING_DIRECTORY ${JSONCPP_DIR}
    COMMAND ${CMAKE_COMMAND} -B out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DJSONCPP_WITH_TESTS=OFF
    COMMAND ${CMAKE_COMMAND} --build out/${MACHINE}
)

# llhttp
add_custom_target(
    FLECS.external.llhttp.build
    WORKING_DIRECTORY ${LLHTTP_DIR}
    COMMAND HOME=$(pwd) npm install
    COMMAND ${CMAKE_MAKE_PROGRAM} PREFIX=out/${MACHINE} -B install
    COMMAND ${CMAKE_MAKE_PROGRAM} PREFIX=out/${MACHINE} clean
)

# mosquitto
add_custom_target(
    FLECS.external.mosquitto.build
    WORKING_DIRECTORY ${MOSQUITTO_DIR}
    COMMAND ${CMAKE_COMMAND} -B out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DDOCUMENTATION=NO
    COMMAND ${CMAKE_COMMAND} --build out/${MACHINE}
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
    COMMAND ${CMAKE_MAKE_PROGRAM} install
    COMMAND ${CMAKE_MAKE_PROGRAM} clean
)

# yaml-cpp
add_custom_target(
    FLECS.external.yaml-cpp.build
    WORKING_DIRECTORY ${YAMLCPP_DIR}
    COMMAND ${CMAKE_COMMAND} -B out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DYAML_CPP_BUILD_TESTS=NO
    COMMAND ${CMAKE_COMMAND} --build out/${MACHINE}
)

# zlib
add_custom_target(
    FLECS.external.yaml-cpp.zlib
    WORKING_DIRECTORY ${ZLIB_DIR}
    COMMAND ${CMAKE_COMMAND} -B out/${MACHINE} -DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE} -DBUILD_SHARED_LIBS=NO
    COMMAND ${CMAKE_COMMAND} --build out/${MACHINE}
)

# imported libraries
# cURL
add_library(FLECS.external.curl STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.curl PROPERTY IMPORTED_LOCATION ${CURL_DIR}/out/${MACHINE}/lib/libcurl.a)
set_property(TARGET FLECS.external.curl PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CURL_DIR}/include)
target_link_libraries(FLECS.external.curl INTERFACE
    FLECS.external.ssl
    FLECS.external.crypto
    FLECS.external.zlib
)

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

# JsonCpp
add_library(FLECS.external.jsoncpp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.jsoncpp PROPERTY IMPORTED_LOCATION ${JSONCPP_DIR}/out/${MACHINE}/lib/libjsoncpp.a)
set_property(TARGET FLECS.external.jsoncpp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${JSONCPP_DIR}/include)

# llhttp
add_library(FLECS.external.llhttp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.llhttp PROPERTY IMPORTED_LOCATION ${LLHTTP_DIR}/out/${MACHINE}/lib/libllhttp.a)
set_property(TARGET FLECS.external.llhttp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${LLHTTP_DIR}/out/${MACHINE}/include)

# OpenSSL
add_library(FLECS.external.crypto STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.crypto PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${MACHINE}/lib/libcrypto.a)
set_property(TARGET FLECS.external.crypto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${MACHINE}/include)
target_link_libraries(FLECS.external.crypto INTERFACE
    dl
)

add_library(FLECS.external.ssl STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.ssl PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${MACHINE}/lib/libssl.a)
set_property(TARGET FLECS.external.ssl PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${MACHINE}/include)

# SQLite3
add_library(FLECS.external.sqlite3 STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.sqlite3 PROPERTY IMPORTED_LOCATION ${SQLITE3_DIR}/out/${MACHINE}/lib/libsqlite3.a)
set_property(TARGET FLECS.external.sqlite3 PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${SQLITE3_DIR})
target_link_libraries(FLECS.external.sqlite3 INTERFACE
    pthread
)

# yaml-cpp
add_library(FLECS.external.yaml-cpp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY IMPORTED_LOCATION ${YAMLCPP_DIR}/out/${MACHINE}/libyaml-cpp.a)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${YAMLCPP_DIR}/include)

# zlib
add_library(FLECS.external.zlib STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.zlib PROPERTY IMPORTED_LOCATION ${ZLIB_DIR}/out/${MACHINE}/libz.a)
set_property(TARGET FLECS.external.zlib PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${ZLIB_DIR})