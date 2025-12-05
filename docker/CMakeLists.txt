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
    install(
        FILES ${CMAKE_SOURCE_DIR}/docker/fs/entrypoint.sh
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs
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
    install(DIRECTORY ${CMAKE_SOURCE_DIR}/pkg/fs/usr/share/flecs/auth/
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/share/flecs/auth
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    # Create log directory for nginx (floxy), as nginx itself can not
    install(
        DIRECTORY DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/var/log/floxy/
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    install(
        FILES ${CMAKE_CURRENT_BINARY_DIR}/../initial_auth_provider.tar
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib/flecs/auth/
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
endfunction()

set(FENCE_DOWNLOAD_CODE [[
    set(ARCH @ARCH@)
    message(STATUS "Building for architecture ${ARCH}")
    set(FENCE_REMOTE_PATH "https://flecs.blob.core.windows.net/flecs-dl/tech.flecs.fence/{1}/flecs-export-tech.flecs.fence_{1}_{2}.tar")
    message(STATUS "Determining latest fence version")
    execute_process(
        COMMAND curl -s --head https://github.com/FLECS-Technologies/apps-tech.flecs.fence/releases/latest
        RESULT_VARIABLE curl_exit_code
        OUTPUT_VARIABLE curl_headers
        ERROR_VARIABLE curl_error
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
    if(NOT curl_exit_code EQUAL 0)
        message(FATAL_ERROR "curl exited with non-zero exit code ${curl_exit_code}: ${curl_error}")
    endif()
    string(REPLACE "\r\n" "\n" curl_headers ${curl_headers})
    string(REGEX MATCH "[Ll]ocation:[ \t]*([^\n]+)" location_header "${curl_headers}")
    string(REGEX REPLACE "^.*/tag/(.+)$" "\\1" fence_tag "${location_header}")
    if(fence_tag STREQUAL "")
        message(FATAL_ERROR "could not extract latest fence tag from ${location_header}")
    endif()
    message(STATUS "Latest fence version: ${fence_tag}")
    string(REPLACE "{1}" "${fence_tag}" FENCE_REMOTE_PATH "${FENCE_REMOTE_PATH}")
    string(REPLACE "{2}" "${ARCH}" FENCE_REMOTE_PATH "${FENCE_REMOTE_PATH}")
    message(STATUS "Downloading fence from ${FENCE_REMOTE_PATH}")
    file(DOWNLOAD
        ${FENCE_REMOTE_PATH}
        ${CMAKE_CURRENT_LIST_DIR}/initial_auth_provider.tar
        STATUS fence_download_status
    )
    list(GET fence_download_status 0 fence_download_result)
    list(GET fence_download_status 1 fence_download_message)
    if(NOT fence_download_result EQUAL 0)
        message(FATAL_ERROR "fence download failed with status ${fence_download_result}: ${fence_download_message}")
    endif()]]
)
string(CONFIGURE "${FENCE_DOWNLOAD_CODE}" FENCE_DOWNLOAD_CODE_CONFIGURED @ONLY)
install(
    CODE "${FENCE_DOWNLOAD_CODE_CONFIGURED}"
    COMPONENT docker
    EXCLUDE_FROM_ALL
)

add_subdirectory(flecs)
add_subdirectory(flecs-slim)
