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

add_library(util.archive STATIC
    archive.h
    impl/archive_entry.h
    impl/base_archive.h
    impl/base_common.h
    impl/read_archive.h
    impl/write_archive.h
    impl/write_base.h
    impl/write_disk.h
    src/archive.cpp
    src/impl/archive_entry.cpp
    src/impl/base_archive.cpp
    src/impl/base_common.cpp
    src/impl/read_archive.cpp
    src/impl/write_archive.cpp
    src/impl/write_base.cpp
    src/impl/write_disk.cpp
)

target_link_libraries(util.archive PRIVATE
    LibArchive::LibArchive
)

add_subdirectory(test)
