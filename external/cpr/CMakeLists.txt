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

include(FetchContent)

FetchContent_Declare(
    cpr
    GIT_REPOSITORY https://github.com/libcpr/cpr/
    GIT_TAG        db351ff
)

set_property(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    APPEND
    PROPERTY COMPILE_OPTIONS -Wno-error -fPIC
)

set (BUILD_SHARED_LIBS NO CACHE BOOL "" FORCE)
set (OPENSSL_ROOT_DIR ${OPENSSL_DIR}/out/${MACHINE} CACHE PATH "" FORCE)
set (OPENSSL_INCLUDE_DIR ${OPENSSL_DIR}/out/${MACHINE}/include CACHE PATH "" FORCE)

FetchContent_MakeAvailable(cpr)

add_library(FLECS.external.cpr INTERFACE)
target_link_libraries(FLECS.external.cpr INTERFACE cpr::cpr)
