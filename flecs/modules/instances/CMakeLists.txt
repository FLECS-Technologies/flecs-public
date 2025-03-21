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

add_library(modules.instances.types STATIC
    src/types/instance_config.cpp
    src/types/instance_id.cpp
    src/types/instance_status.cpp
    src/types/instance.cpp
    types/instance_config.h
    types/instance_id.h
    types/instance_status.h
    types/instance.h
    types.h
)

target_link_libraries(modules.instances.types PUBLIC
    util.usb
    common
)

target_link_libraries(modules.instances.types INTERFACE
    modules.apps.types
)

flecs_add_module(
    MODULE_NAME
        instances
    ADDITIONAL_SOURCES
        src/impl/instances_impl.cpp
    ADDITIONAL_HEADERS
        impl/instances_impl.h
    LIBS_PUBLIC
        common
        modules.instances.types
        modules.jobs.types
    LIBS_PRIVATE
        modules.deployments.types
        modules.apps
        modules.jobs
        util.datetime
        util.process
        util.network
)

add_subdirectory(test)
