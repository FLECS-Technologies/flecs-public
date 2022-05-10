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

flecs_add_module(
    MODULE_NAME app_manager
    ADDITIONAL_SOURCES
        src/private/app_manager_private.cpp
        src/private/app_install.cpp
        src/private/app_instances.cpp
        src/private/app_list.cpp
        src/private/app_sideload.cpp
        src/private/app_uninstall.cpp
        src/private/app_versions.cpp
        src/private/instance_config.cpp
        src/private/instance_create.cpp
        src/private/instance_delete.cpp
        src/private/instance_details.cpp
        src/private/instance_log.cpp
        src/private/instance_start.cpp
        src/private/instance_stop.cpp
    ADDITIONAL_HEADERS private/app_manager_private.h
    LIBS_PUBLIC FLECS.daemon.db
    LIBS_PRIVATE FLECS.external.cpr FLECS.external.json FLECS.daemon.app
)
