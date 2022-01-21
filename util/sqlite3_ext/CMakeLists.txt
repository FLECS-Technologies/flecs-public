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

project(FLECS.util.sqlite3_ext)

set(SRC_ARCHIVE
    src/sqlite3_db.cpp
    src/sqlite3_table.cpp
)

set (HEADER_ARCHIVE
    sqlite3_column.h
    sqlite3_db.h
    sqlite3_primary.h
    sqlite3_statements.h
    sqlite3_table.h
)

add_library(FLECS.util.sqlite3_ext STATIC ${SRC_ARCHIVE} ${HEADER_ARCHIVE})

target_link_libraries(FLECS.util.sqlite3_ext PUBLIC FLECS.external.sqlite3 stdc++fs)

target_include_directories(FLECS.util.sqlite3_ext PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)
