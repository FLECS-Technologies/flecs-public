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

set(SRC_ARCHIVE
    src/datetime.cpp
)

set (HEADER_ARCHIVE
    datetime.h
)

add_library(util.datetime OBJECT ${SRC_ARCHIVE} ${HEADER_ARCHIVE})

target_compile_definitions(util.datetime PUBLIC
    -D_FILE_OFFSET_BITS=64
    -D_TIME_BITS=64
)

add_subdirectory(test)
