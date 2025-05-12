# Copyright 2021-2024 FLECS Technologies GmbH
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

function(stage_docker_fs)
    install(
        FILES ${CMAKE_BINARY_DIR}/flecsd
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/bin
        PERMISSIONS
            OWNER_READ OWNER_WRITE OWNER_EXECUTE
            GROUP_READ GROUP_EXECUTE
            WORLD_READ WORLD_EXECUTE
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    install(
        FILES ${CMAKE_SOURCE_DIR}/flecsd/scripts/flecs-update-hosts.sh
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/bin
        PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE
        PERMISSIONS GROUP_READ GROUP_EXECUTE
        PERMISSIONS WORLD_READ WORLD_EXECUTE
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    install(FILES ${CMAKE_SOURCE_DIR}/pkg/fs/etc/nginx/floxy.conf
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/etc/nginx
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    # Create log directory for nginx (floxy), as nginx itself can not
    install(
        DIRECTORY DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/var/log/floxy/
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
endfunction()

add_subdirectory(flecs)
add_subdirectory(flecs-slim)
