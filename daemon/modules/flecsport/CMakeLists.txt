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

flecs_add_module(
    MODULE_NAME flecsport
    ADDITIONAL_SOURCES
    src/export_manifest.cpp src/impl/flecsport_impl.cpp
    ADDITIONAL_HEADERS
    export_manifest.h impl/flecsport_impl.h
    LIBS_PUBLIC daemon.common
    LIBS_PRIVATE util.archive
)

add_library(daemon.modules.flecsport.mocked_modules STATIC
    src/flecsport.cpp
    src/impl/flecsport_impl.cpp
    flecsport.h
    impl/flecsport_impl.h
)

target_link_libraries(
    daemon.modules.flecsport.mocked_modules PUBLIC
    GTest::gmock
    daemon.modules.factory
    daemon.modules.module_base
)

target_compile_definitions(
    daemon.modules.flecsport.mocked_modules PRIVATE
    -DFLECS_MOCK_MODULES
)
