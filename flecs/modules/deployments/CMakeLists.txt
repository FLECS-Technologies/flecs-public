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

add_library(modules.deployments.types STATIC
    src/types/deployment.cpp
    src/types/deployment_compose.cpp
    src/types/deployment_docker.cpp
    types/deployment.h
    types/deployment_compose.h
    types/deployment_docker.h
    types.h
)

target_link_libraries(modules.deployments.types PRIVATE modules.floxy)

flecs_add_module(
    MODULE_NAME
        deployments
    ADDITIONAL_SOURCES
        src/impl/deployments_impl.cpp
    ADDITIONAL_HEADERS
        impl/deployments_impl.h
    LIBS_PUBLIC
        common.app.manifest
        modules.deployments.types
        modules.instances.types
        util.network
)

if(FLECS_BUILD_TESTS)
    add_library(modules.deployments.types.mocked_modules STATIC
        src/types/deployment.cpp
        src/types/deployment_compose.cpp
        src/types/deployment_docker.cpp
        types/deployment.h
        types/deployment_compose.h
        types/deployment_docker.h
        types.h
    )
    target_link_libraries(modules.deployments.types.mocked_modules PRIVATE
        modules.floxy.mocked_modules
        util.network
        util.process
    )
    target_compile_definitions(
        modules.deployments.types.mocked_modules PRIVATE
        -DFLECS_MOCK_MODULES
    )
endif()

add_subdirectory(test)
