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

set(CMAKE_ARGS -G Ninja -B build/${ARCH} -DCMAKE_INSTALL_PREFIX=${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH} -DCMAKE_TOOLCHAIN_FILE=${TOOLCHAIN_FILE} -DCMAKE_CXX_STANDARD=17 -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE})

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libdocker.a
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/libdocker
    COMMAND ${CMAKE_COMMAND} ${CMAKE_ARGS}
    COMMAND ${CMAKE_COMMAND} --build build/${ARCH}
    COMMAND ${CMAKE_COMMAND} --install build/${ARCH}
)
add_custom_target(external.libdocker.build DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libdocker.a)

add_library(external.libdocker INTERFACE)
set_property(TARGET external.libdocker PROPERTY INTERFACE_LINK_LIBRARIES ${CMAKE_CURRENT_SOURCE_DIR}/out/${ARCH}/lib/libdocker.a gcov)
set_property(TARGET external.libdocker PROPERTY INTERFACE_INCLUDE_DIRECTORIES ${CMAKE_CURRENT_SOURCE_DIR}/libdocker ${CMAKE_CURRENT_SOURCE_DIR}/libdocker/libdocker)
