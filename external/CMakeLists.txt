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

project(external)

# define source directories
set(CMAKE_ARGS -B build/${ARCH} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH} -DCMAKE_TOOLCHAIN_FILE=${TOOLCHAIN_FILE} -DCMAKE_CXX_STANDARD=17 -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_BUILD_TYPE=Release)
set(CPR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/cpr-1.10.2)
set(CROW_DIR ${CMAKE_CURRENT_SOURCE_DIR}/crow-1.0+5)
set(GTEST_DIR ${CMAKE_CURRENT_SOURCE_DIR}/googletest-1.13.0)
set(JSON_DIR ${CMAKE_CURRENT_SOURCE_DIR}/json-3.11.2)
set(LIBARCHIVE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/libarchive-3.6.2)
set(MOSQUITTO_DIR ${CMAKE_CURRENT_SOURCE_DIR}/mosquitto-2.0.15)
set(OPENSSL_DIR ${CMAKE_CURRENT_SOURCE_DIR}/openssl-3.1.0)
set(PYBIND11_DIR ${CMAKE_CURRENT_SOURCE_DIR}/pybind11-2.10.3)
set(YAMLCPP_DIR ${CMAKE_CURRENT_SOURCE_DIR}/yaml-cpp-0.7.0)
set(ZENOH_C_DIR ${CMAKE_CURRENT_SOURCE_DIR}/zenoh-c-0.6.0-beta.1)
set(ZLIB_DIR ${CMAKE_CURRENT_SOURCE_DIR}/zlib-1.2.13)

# custom build rules
# cpr
add_custom_target(
    external.cpr.build
    WORKING_DIRECTORY ${CPR_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DBUILD_SHARED_LIBS=NO -DOPENSSL_ROOT_DIR=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH} -DOPENSSL_INCLUDE_DIR=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# crow
add_custom_target(
    external.crow.build
    WORKING_DIRECTORY ${CROW_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DCROW_BUILD_EXAMPLES=OFF -DCROW_BUILD_TESTS=OFF
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# gtest
add_custom_target(
    external.gtest.build
    WORKING_DIRECTORY ${GTEST_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# libarchive
add_custom_target(
    external.libarchive.build
    WORKING_DIRECTORY ${LIBARCHIVE_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DENABLE_OPENSSL=OFF -DENABLE_LIBXML2=OFF -DENABLE_EXPAT=OFF
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# libmosquitto
add_custom_target(
    external.mosquitto.build
    WORKING_DIRECTORY ${MOSQUITTO_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DDOCUMENTATION=NO -DWITH_STATIC_LIBRARIES=YES -DWITH_PIC=YES -DWITH_APPS=NO -DWITH_BROKER=NO -DWITH_CLIENTS=NO -DWITH_PLUGINS=NO -DOPENSSL_ROOT_DIR=${OPENSSL_DIR}/out/${ARCH}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# OpenSSL
add_custom_target(
    external.openssl.build
    WORKING_DIRECTORY ${OPENSSL_DIR}
    COMMAND flecs-configure/Configure.${ARCH}.sh ${OPENSSL_DIR} --cross-compile-prefix=${MACHINE}- --prefix=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH} no-shared
    COMMAND make all
    COMMAND make install_sw
    COMMAND make clean
)

# pybind11
add_custom_target(
    external.pybind11.build
    WORKING_DIRECTORY ${PYBIND11_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# yaml-cpp
add_custom_target(
    external.yaml-cpp.build
    WORKING_DIRECTORY ${YAMLCPP_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS} -DYAML_CPP_BUILD_TESTS=NO
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# zlib
add_custom_target(
    external.zlib.build
    WORKING_DIRECTORY ${ZLIB_DIR}
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)

# Zenoh-C
# Use prebuilt

# imported libraries
# cpr
add_library(external.cpr.curl STATIC IMPORTED GLOBAL)
set_property(TARGET external.cpr.curl PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libcurl.a)
target_link_libraries(external.cpr.curl INTERFACE
    external.ssl
    external.crypto
)

add_library(external.cpr.zlib STATIC IMPORTED GLOBAL)
set_property(TARGET external.cpr.zlib PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libz.a)

add_library(external.cpr STATIC IMPORTED GLOBAL)
set_property(TARGET external.cpr PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libcpr.a)
set_property(TARGET external.cpr PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)
target_link_libraries(external.cpr INTERFACE
    external.cpr.curl
    external.cpr.zlib
)

# crow
add_library(external.crow INTERFACE)
target_include_directories(external.crow INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include
    ${Boost_INCLUDE_DIRS}
)

# gtest
add_library(external.gmock_main STATIC IMPORTED GLOBAL)
set_property(TARGET external.gmock_main PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/libgmock_main.a)
set_property(TARGET external.gmock_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

add_library(external.gmock STATIC IMPORTED GLOBAL)
set_property(TARGET external.gmock PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libgmock.a)
set_property(TARGET external.gmock PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

add_library(external.gtest_main STATIC IMPORTED GLOBAL)
set_property(TARGET external.gtest_main PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libgtest_main.a)
set_property(TARGET external.gtest_main PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

add_library(external.gtest STATIC IMPORTED GLOBAL)
set_property(TARGET external.gtest PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libgtest.a)
set_property(TARGET external.gtest PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# nlohmann/json
add_library(external.json INTERFACE)
target_include_directories(external.json INTERFACE
    ${JSON_DIR}/include
)

# libarchive
add_library(external.libarchive STATIC IMPORTED GLOBAL)
set_property(TARGET external.libarchive PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libarchive.a)
target_include_directories(external.libarchive INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)
target_link_libraries(external.libarchive INTERFACE
    external.zlib
)

# libmosquitto
add_library(external.mosquitto STATIC IMPORTED GLOBAL)
set_property(TARGET external.mosquitto PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libmosquitto_static.a)
set_property(TARGET external.mosquitto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)
target_link_libraries(external.mosquitto INTERFACE
    external.ssl
    external.crypto
)

# OpenSSL
add_library(external.crypto STATIC IMPORTED GLOBAL)
set_property(TARGET external.crypto PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libcrypto.a)
set_property(TARGET external.crypto PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

add_library(external.ssl STATIC IMPORTED GLOBAL)
set_property(TARGET external.ssl PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libssl.a)
set_property(TARGET external.ssl PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# pybind11
add_library(external.pybind11 INTERFACE)
target_include_directories(external.pybind11 INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# yaml-cpp
add_library(external.yaml-cpp STATIC IMPORTED GLOBAL)
set_property(TARGET external.yaml-cpp PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libyaml-cpp.a)
set_property(TARGET external.yaml-cpp PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)

# zenoh-c
add_library(external.zenoh-c SHARED IMPORTED GLOBAL)
set_property(TARGET external.zenoh-c PROPERTY IMPORTED_LOCATION ${ZENOH_C_DIR}/out/${ARCH}/lib/libzenohc.so)
set_property(TARGET external.zenoh-c PROPERTY IMPORTED_NO_SONAME TRUE)
set_property(TARGET external.zenoh-c PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${ZENOH_C_DIR}/out/${ARCH}/include)

# zlib
add_library(external.zlib STATIC IMPORTED GLOBAL)
set_property(TARGET external.zlib PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libz.a)
set_property(TARGET external.zlib PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/include)
