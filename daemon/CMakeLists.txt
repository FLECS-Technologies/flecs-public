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

project(flecsd)

add_subdirectory(api)
add_subdirectory(common)
add_subdirectory(modules)

# build executable flecsd
add_executable(daemon
    src/main.cpp
)
set_target_properties(daemon PROPERTIES OUTPUT_NAME flecsd)

# FLECS link libraries
target_link_libraries(daemon PRIVATE
    daemon.api
    daemon.common
    daemon.modules.apps
    daemon.modules.console
    daemon.modules.data_layer
    daemon.modules.device
    daemon.modules.factory
    daemon.modules.flecsport
    daemon.modules.instances
    daemon.modules.jobs
    daemon.modules.manifests
    daemon.modules.module_base
    daemon.modules.system
    daemon.modules.version
    util.datetime
    util.process
    util.signal_handler
    yaml-cpp::yaml-cpp
)

# Get all propreties that cmake supports
if(NOT CMAKE_PROPERTY_LIST)
    execute_process(COMMAND cmake --help-property-list OUTPUT_VARIABLE CMAKE_PROPERTY_LIST)

    # Convert command output into a CMake list
    string(REGEX REPLACE ";" "\\\\;" CMAKE_PROPERTY_LIST "${CMAKE_PROPERTY_LIST}")
    string(REGEX REPLACE "\n" ";" CMAKE_PROPERTY_LIST "${CMAKE_PROPERTY_LIST}")
    list(REMOVE_DUPLICATES CMAKE_PROPERTY_LIST)
endif()

function(print_properties)
    message("CMAKE_PROPERTY_LIST = ${CMAKE_PROPERTY_LIST}")
endfunction()

function(print_target_properties target)
    if(NOT TARGET ${target})
      message(STATUS "There is no target named '${target}'")
      return()
    endif()

    foreach(property ${CMAKE_PROPERTY_LIST})
        string(REPLACE "<CONFIG>" "${CMAKE_BUILD_TYPE}" property ${property})

        # Fix https://stackoverflow.com/questions/32197663/how-can-i-remove-the-the-location-property-may-not-be-read-from-target-error-i
        #if(property STREQUAL "LOCATION" OR property MATCHES "^LOCATION_" OR property MATCHES "_LOCATION$")
        #    continue()
        #endif()

        get_property(was_set TARGET ${target} PROPERTY ${property} SET)
        if(was_set)
            get_target_property(value ${target} ${property})
            message("${target} ${property} = ${value}")
        endif()
    endforeach()
endfunction()

#print_target_properties(LibArchive::LibArchive)
#message(FATAL_ERROR)

# install executable flecsd for packaging
install(
    TARGETS daemon RUNTIME
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/opt/flecs/bin
)

# install shared objects for packaging
install(
    TARGETS daemon.lib flunder.shared LIBRARY
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/usr/local/lib/${MACHINE}
)

# Alias targets cannot be installed, so retrieve the aliased targets first
get_property(curl_target TARGET CURL::libcurl PROPERTY ALIASED_TARGET)
get_property(zenohc_target TARGET zenohc::lib PROPERTY ALIASED_TARGET)
install(
    IMPORTED_RUNTIME_ARTIFACTS
        cpr::cpr
        ${curl_target}
        yaml-cpp::yaml-cpp
        ${zenohc_target}
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/usr/local/lib/${MACHINE}
)

# These libs are of type "UNKNOWN", and cannot be installed with IMPORTED_RUNTIME_ARTIFACTS
get_property(openssl_crypto_lib TARGET OpenSSL::Crypto PROPERTY IMPORTED_LOCATION)
get_property(openssl_ssl_lib TARGET OpenSSL::SSL PROPERTY IMPORTED_LOCATION)
install(
    CODE "file (
            INSTALL ${openssl_crypto_lib} ${openssl_ssl_lib} ${pkgcfg_lib_libusb_usb-1.0}
            DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/usr/local/lib/${MACHINE}
            FOLLOW_SYMLINK_CHAIN
        )"
)

# Installing LibArchive this way won't work, however, as:
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
            DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/usr/local/lib/${MACHINE}
        )"
)

# install helper scripts for packaging
install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/scripts/
    DESTINATION ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/docker/fs/opt/flecs/bin
    USE_SOURCE_PERMISSIONS
)

# configure Docker image flecs
set(DOCKER_REGISTRY "flecspublic.azurecr.io")
set(DOCKER_IMAGE "flecs")
set(DOCKER_TAG "${CORE_VERSION}")
include(flecs-docker)

# configure Docker image flecs-slim
set(DOCKER_REGISTRY "flecspublic.azurecr.io")
set(DOCKER_IMAGE "flecs-slim")
set(DOCKER_TAG "${CORE_VERSION}")
include(flecs-docker)

# configure package
set(PACKAGE flecs)
set(PACKAGE_VERSION ${CORE_VERSION})
set(PACKAGE_DESC FLECS)
set(PACKAGE_DEPENDS "docker-ce (>= 20.10.5) | docker.io (>= 20.10.5)")

add_custom_target(
    flecs_deb-pkg-copy
    WORKING_DIRECTORY ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/pkg
    COMMAND pwd
    COMMAND sed -i 's/^DOCKER_TAG.*/DOCKER_TAG=${DOCKER_TAG}/g' */opt/flecsd/bin/flecsd.sh
)

include(flecs-package)