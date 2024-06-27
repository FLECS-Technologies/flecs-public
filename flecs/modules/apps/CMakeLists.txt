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

add_library(modules.apps.types STATIC
    src/types/app_key.cpp
    src/types/app_name.cpp
    src/types/app_status.cpp
    src/types/app.cpp
    types/app_key.h
    types/app_name.h
    types/app_status.h
    types/app.h
    types.h
)

flecs_add_module(
    MODULE_NAME
        apps
    ADDITIONAL_SOURCES
        src/impl/apps_impl.cpp
    ADDITIONAL_HEADERS
        impl/apps_impl.h
    LIBS_PUBLIC
        common
        modules.apps.types
        modules.instances.types
        modules.manifests
    LIBS_PRIVATE
        cpr::cpr
        util.usb
        yaml-cpp::yaml-cpp
        modules.flecsport
)

if(FLECS_BUILD_TESTS)
    add_library(modules.apps.mocked_modules STATIC
        src/apps.cpp
        src/impl/apps_impl.cpp
        apps.h
        impl/apps_impl.h
    )

    target_link_libraries(
        modules.apps.mocked_modules PUBLIC
        GTest::gmock
        common
        modules.factory
        modules.module_base
    )

    target_compile_definitions(
        modules.apps.mocked_modules PRIVATE
        -DFLECS_MOCK_MODULES
    )
endif()

add_subdirectory(test)
