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

function(install_external lib dest cmp)
    # try to resolve aliased target
    get_property(aliased_target TARGET ${lib} PROPERTY ALIASED_TARGET)
    if("${aliased_target}" STREQUAL "")
        # use original target if not aliased
        set(aliased_target ${lib})
    endif()

    get_property(library_type TARGET ${lib} PROPERTY TYPE)
    if ("${library_type}" STREQUAL "UNKNOWN_LIBRARY")
        get_property(library_location TARGET ${lib} PROPERTY IMPORTED_LOCATION)
        if ("${library_location}" STREQUAL "")
            message(FATAL_ERROR "Could not determine IMPORTED_LOCATION of ${lib}")
        endif()
        install(
            CODE "file (
                    INSTALL ${library_location}
                    DESTINATION ${dest}
                    FOLLOW_SYMLINK_CHAIN
                )"
            COMPONENT ${cmp}
            EXCLUDE_FROM_ALL
        )
    else()
        get_property(library_artifacts TARGET ${lib} PROPERTY IMPORTED_RUNTIME_ARTIFACTS)
        install(
            IMPORTED_RUNTIME_ARTIFACTS ${aliased_target}
            DESTINATION ${dest}
            COMPONENT ${cmp}
            EXCLUDE_FROM_ALL
        )
    endif()
endfunction()

function(stage_docker_fs)
    install(
        TARGETS daemon RUNTIME
        DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/bin
        COMPONENT docker
        EXCLUDE_FROM_ALL
    )
    install(
        FILES ${CMAKE_SOURCE_DIR}/flecs/scripts/flecs-update-hosts.sh
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
    install_external(CURL::libcurl ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib docker)
    install_external(Libusb::Libusb ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib docker)
    install_external(OpenSSL::Crypto ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib docker)
    install_external(OpenSSL::SSL ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib docker)
    install_external(yaml-cpp::yaml-cpp ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib docker)

    # Installing LibArchive this way won't work, as:
    #   lrwxrwxrwx libarchive.so -> libarchive.so.13.x.y
    #   lrwxrwxrwx libarchive.so.13 -> libarchive.so.13.x.y
    #   -rwxr-xr-x libarchive.so.13.x.y
    # As LibArchive's IMPORTED_LOCATION points to the .so file, libarchive.so and
    # libarchive.so.13.x.y would be installed, while libarchive.so.13 is the one we
    # need during runtime.
    get_property(archive_lib TARGET LibArchive::LibArchive PROPERTY IMPORTED_LOCATION)
    file(GLOB archive_libs "${archive_lib}*")
    install(
        CODE "file (
                INSTALL ${archive_libs}
                DESTINATION ${CMAKE_CURRENT_BINARY_DIR}/fs/usr/local/lib
            )"
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
