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
    src/process.cpp
)

set (HEADER_ARCHIVE
    posix_spawn.h
    process.h
)

add_library(util.process STATIC ${SRC_ARCHIVE} ${HEADER_ARCHIVE})

add_subdirectory(test)
