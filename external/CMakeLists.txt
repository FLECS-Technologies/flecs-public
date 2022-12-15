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

# define source directories
set(CMAKE_ARGS -B build/${ARCH} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH} -DCMAKE_TOOLCHAIN_FILE=${TOOLCHAIN_FILE} -DCMAKE_CXX_STANDARD=17 -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_BUILD_TYPE=Release)
set(CPR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/cpr-1.9.3)
set(CROW_DIR ${CMAKE_CURRENT_SOURCE_DIR}/crow-1.0+5)
set(GTEST_DIR ${CMAKE_CURRENT_SOURCE_DIR}/gtest-1.12.1)
set(JSON_DIR ${CMAKE_CURRENT_SOURCE_DIR}/json-3.11.2)
set(LIBARCHIVE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/libarchive-3.6.2)
set(LIBSODIUM_DIR ${CMAKE_CURRENT_SOURCE_DIR}/libsodium-1.0.18)
set(MOSQUITTO_DIR ${CMAKE_CURRENT_SOURCE_DIR}/mosquitto-2.0.15)
set(OPENSSL_DIR ${CMAKE_CURRENT_SOURCE_DIR}/openssl-1.1.1s)
set(PYBIND11_DIR ${CMAKE_CURRENT_SOURCE_DIR}/pybind11-2.10.1)
set(SQLITE3_DIR ${CMAKE_CURRENT_SOURCE_DIR}/sqlite-3.39.3)
set(YAMLCPP_DIR ${CMAKE_CURRENT_SOURCE_DIR}/yaml-cpp-0.7.0)
set(ZENOH_C_DIR ${CMAKE_CURRENT_SOURCE_DIR}/zenoh-c-0.6.0-beta.1)

# custom build rules
# cpr
add_custom_target(
    FLECS.external.cpr.build
    WORKING_DIRECTORY ${CPR_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DBUILD_SHARED_LIBS=NO -DOPENSSL_ROOT_DIR=${OPENSSL_DIR}/out/${ARCH} -DOPENSSL_INCLUDE_DIR=${OPENSSL_DIR}/out/${ARCH}/include
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# gtest
add_custom_target(
    FLECS.external.gtest.build
    WORKING_DIRECTORY ${GTEST_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# libarchive
add_custom_target(
    FLECS.external.libarchive.build
    WORKING_DIRECTORY ${LIBARCHIVE_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# libmosquitto
add_custom_target(
    FLECS.external.mosquitto.build
    WORKING_DIRECTORY ${MOSQUITTO_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DDOCUMENTATION=NO -DWITH_STATIC_LIBRARIES=YES -DWITH_PIC=YES -DWITH_APPS=NO -DWITH_BROKER=NO -DWITH_CLIENTS=NO -DWITH_PLUGINS=NO -DOPENSSL_ROOT_DIR=${OPENSSL_DIR}/out/${ARCH}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# libsodium
add_custom_target(
    FLECS.external.libsodium.build
    WORKING_DIRECTORY ${LIBSODIUM_DIR}
    COMMAND ./configure --prefix=${LIBSODIUM_DIR}/out/${ARCH} --host=${CROSS_COMPILE}
    COMMAND ${CMAKE_MAKE_PROGRAM} install
    COMMAND ${CMAKE_MAKE_PROGRAM} clean
)

# OpenSSL
add_custom_target(
    FLECS.external.openssl.build
    WORKING_DIRECTORY ${OPENSSL_DIR}
    COMMAND flecs-configure/Configure.${ARCH}.sh ${OPENSSL_DIR} --cross-compile-prefix=${MACHINE}- --prefix=${OPENSSL_DIR}/out/${ARCH} no-shared
    COMMAND make all
    COMMAND make install_sw
    COMMAND make clean
)

# pybind11
add_custom_target(
    FLECS.external.pybind11.build
    WORKING_DIRECTORY ${PYBIND11_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# sqlite3
add_custom_target(
    FLECS.external.sqlite3.build
    WORKING_DIRECTORY ${SQLITE3_DIR}
    COMMAND autoreconf
    COMMAND ./configure --prefix=${SQLITE3_DIR}/out/${ARCH} --host=${CROSS_COMPILE}
    COMMAND make install
    COMMAND make clean
)

# yaml-cpp
add_custom_target(
    FLECS.external.yaml-cpp.build
    WORKING_DIRECTORY ${YAMLCPP_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DYAML_CPP_BUILD_TESTS=NO
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# Zenoh-C
# Use prebuilt

# imported libraries
# cpr
add_library(FLECS.external.cpr.curl STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.cpr.curl PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libcurl.a)
target_link_libraries(FLECS.external.cpr.curl INTERFACE
    FLECS.external.ssl
    FLECS.external.crypto
)

add_library(FLECS.external.cpr.zlib STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.cpr.zlib PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libz.a)

add_library(FLECS.external.cpr STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.cpr PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libcpr.a)
set_property(TARGET FLECS.external.cpr PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)
target_link_libraries(FLECS.external.cpr INTERFACE
    FLECS.external.cpr.curl
    FLECS.external.cpr.zlib
)

# crow
add_library(FLECS.external.crow INTERFACE)
target_include_directories(FLECS.external.crow INTERFACE
    ${CROW_DIR}/include
    ${Boost_INCLUDE_DIRS}
)

# gtest
add_library(FLECS.external.gmock_main STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gmock_main PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/out/amd64/lib/libgmock_main.a)
set_property(TARGET FLECS.external.gmock_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR}/out/amd64/include)

add_library(FLECS.external.gmock STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gmock PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/out/amd64/lib/libgmock.a)
set_property(TARGET FLECS.external.gmock PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR}/out/amd64/include)

add_library(FLECS.external.gtest_main STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gtest_main PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/out/amd64/lib/libgtest_main.a)
set_property(TARGET FLECS.external.gtest_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR}/out/amd64/include)

add_library(FLECS.external.gtest STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.gtest PROPERTY IMPORTED_LOCATION ${GTEST_DIR}/out/amd64/lib/libgtest.a)
set_property(TARGET FLECS.external.gtest PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${GTEST_DIR}/out/amd64/include)

# nlohmann/json
add_library(FLECS.external.json INTERFACE)
target_include_directories(FLECS.external.json INTERFACE
    ${JSON_DIR}/include
)

# libarchive
add_library(FLECS.external.libarchive STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.libarchive PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libarchive.a)
target_include_directories(FLECS.external.libarchive INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# libmosquitto
add_library(FLECS.external.mosquitto STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.mosquitto PROPERTY IMPORTED_LOCATION ${MOSQUITTO_DIR}/out/${ARCH}/lib/libmosquitto_static.a)
set_property(TARGET FLECS.external.mosquitto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${MOSQUITTO_DIR}/include)
target_link_libraries(FLECS.external.mosquitto INTERFACE
    FLECS.external.ssl
    FLECS.external.crypto
)

# libsodium
add_library(FLECS.external.libsodium STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.libsodium PROPERTY IMPORTED_LOCATION ${LIBSODIUM_DIR}/out/${ARCH}/lib/libsodium.a)
set_property(TARGET FLECS.external.libsodium PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${LIBSODIUM_DIR}/out/${ARCH}/include)

# OpenSSL
add_library(FLECS.external.crypto STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.crypto PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${ARCH}/lib/libcrypto.a)
set_property(TARGET FLECS.external.crypto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${ARCH}/include)

add_library(FLECS.external.ssl STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.ssl PROPERTY IMPORTED_LOCATION ${OPENSSL_DIR}/out/${ARCH}/lib/libssl.a)
set_property(TARGET FLECS.external.ssl PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${OPENSSL_DIR}/out/${ARCH}/include)

# pybind11
add_library(FLECS.external.pybind11 INTERFACE)
target_include_directories(FLECS.external.pybind11 INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# sqlite3
add_library(FLECS.external.sqlite3 STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.sqlite3 PROPERTY IMPORTED_LOCATION ${SQLITE3_DIR}/out/${ARCH}/lib/libsqlite3.a)
set_property(TARGET FLECS.external.sqlite3 PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${SQLITE3_DIR})

# yaml-cpp
add_library(FLECS.external.yaml-cpp STATIC IMPORTED GLOBAL)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY IMPORTED_LOCATION ${YAMLCPP_DIR}/out/${ARCH}/lib/libyaml-cpp.a)
set_property(TARGET FLECS.external.yaml-cpp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${YAMLCPP_DIR}/include)

# zenoh-c
add_library(FLECS.external.zenoh-c SHARED IMPORTED GLOBAL)
set_property(TARGET FLECS.external.zenoh-c PROPERTY IMPORTED_LOCATION ${ZENOH_C_DIR}/out/${ARCH}/lib/libzenohc.so)
set_property(TARGET FLECS.external.zenoh-c PROPERTY IMPORTED_NO_SONAME TRUE)
set_property(TARGET FLECS.external.zenoh-c PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${ZENOH_C_DIR}/out/${ARCH}/include)
