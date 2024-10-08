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
    MODULE_NAME floxy
    ADDITIONAL_HEADERS impl/floxy_impl.h
    ADDITIONAL_SOURCES src/impl/floxy_impl.cpp
    LIBS_PRIVATE util.process modules.instances modules.apps util.process common
    LIBS_PUBLIC modules.instances.types
)

if(FLECS_BUILD_TESTS)
    add_library(modules.floxy.mocked_modules STATIC
        src/floxy.cpp
        src/impl/floxy_impl.cpp
        floxy.h
        impl/floxy_impl.h
    )

    target_link_libraries(
        modules.floxy.mocked_modules PUBLIC
        GTest::gmock
        modules.factory
        modules.module_base
    )

    target_compile_definitions(
        modules.floxy.mocked_modules PRIVATE
        -DFLECS_MOCK_MODULES
    )
endif()

#add_subdirectory(test)
