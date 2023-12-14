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
    MODULE_NAME manifests
    ADDITIONAL_SOURCES
    src/impl/manifests_impl.cpp
    ADDITIONAL_HEADERS
    impl/manifests_impl.h
    LIBS_PRIVATE
        cpr::cpr
        daemon.modules.console
        daemon.modules.device
        daemon.common.app.manifest
        modules.apps.types
)

add_library(daemon.modules.manifests.mocked_modules STATIC
    src/manifests.cpp
    src/impl/manifests_impl.cpp
    manifests.h
    impl/manifests_impl.h
)

target_link_libraries(
    daemon.modules.manifests.mocked_modules PUBLIC
    GTest::gmock
    daemon.common
    daemon.modules.factory
    daemon.modules.module_base
)

target_compile_definitions(
    daemon.modules.manifests.mocked_modules PRIVATE
    -DFLECS_MOCK_MODULES
)

add_subdirectory(test)
