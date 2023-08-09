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

# external 3rd party depencies
# cpr
add_library(external.curl INTERFACE)
target_link_libraries(external.curl INTERFACE
    curl
    external.ssl
    external.crypto
)

add_library(external.cpr INTERFACE)
target_link_libraries(external.cpr INTERFACE
    cpr
    $<TARGET_PROPERTY:external.curl,INTERFACE_LINK_LIBRARIES>
    $<TARGET_PROPERTY:external.zlib,INTERFACE_LINK_LIBRARIES>
)

# gtest
add_library(external.gmock_main INTERFACE)
target_link_libraries(external.gmock_main INTERFACE
    gmock_main
)
add_library(external.gmock INTERFACE)
target_link_libraries(external.gmock INTERFACE
    gmock
)
add_library(external.gtest_main INTERFACE)
target_link_libraries(external.gtest_main INTERFACE
    gtest_main
)
add_library(external.gtest INTERFACE)
target_link_libraries(external.gtest INTERFACE
    gtest
)

# libarchive
add_library(external.libarchive INTERFACE)
target_link_libraries(external.libarchive INTERFACE
    archive
    $<TARGET_PROPERTY:external.zlib,INTERFACE_LINK_LIBRARIES>
    $<TARGET_PROPERTY:external.crypto,INTERFACE_LINK_LIBRARIES>
)

# libmosquitto
add_library(external.mosquitto INTERFACE)
target_link_libraries(external.mosquitto INTERFACE
    mosquitto_static
    $<TARGET_PROPERTY:external.ssl,INTERFACE_LINK_LIBRARIES>
    $<TARGET_PROPERTY:external.crypto,INTERFACE_LINK_LIBRARIES>
)

# libusb
add_library(external.libusb INTERFACE)
target_link_libraries(external.libusb INTERFACE
    usb-1.0
)

# OpenSSL
add_library(external.crypto INTERFACE)
target_link_libraries(external.crypto INTERFACE
    crypto
)

add_library(external.ssl INTERFACE)
target_link_libraries(external.ssl INTERFACE
    ssl
)

# yaml-cpp
add_library(external.yaml-cpp INTERFACE)
target_link_libraries(external.yaml-cpp INTERFACE
    yaml-cpp
)

# zenoh-c
add_library(external.zenoh-c INTERFACE)
target_link_libraries(external.zenoh-c INTERFACE
    zenohc
)

# zlib
add_library(external.zlib INTERFACE)
target_link_libraries(external.zlib INTERFACE
    z
)
