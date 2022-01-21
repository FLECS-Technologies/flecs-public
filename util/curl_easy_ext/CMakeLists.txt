# Copyright 2021 FLECS Technologies GmbH
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

project(FLECS.util.curl_easy_ext)

set(SRC_ARCHIVE
    src/curl_easy_ext.cpp
)

set (HEADER_ARCHIVE
    curl_easy_ext.h
)

add_library(FLECS.util.curl_easy_ext STATIC ${SRC_ARCHIVE} ${HEADER_ARCHIVE})

target_link_libraries(FLECS.util.curl_easy_ext PUBLIC FLECS.external.curl)

target_include_directories(FLECS.util.curl_easy_ext PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)
