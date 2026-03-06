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

if(NOT DOCKER_TAG)
    message(FATAL_ERROR "Cannot build Docker image without tagname")
endif()

if (DOCKER_VARIANT)
    set(DOCKER_TAG ${DOCKER_TAG}${DOCKER_VARIANT})
endif()

add_custom_command(
    OUTPUT ${DOCKER_IMAGE}${DOCKER_VARIANT}_buildx
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND bash -x ${CMAKE_CURRENT_LIST_DIR}/build-image.sh --arch ${ARCH} --image ${DOCKER_REGISTRY}/${DOCKER_IMAGE} --tag ${DOCKER_TAG} --variant ${DOCKER_VARIANT}
)

add_custom_command(
    OUTPUT ${DOCKER_IMAGE}${DOCKER_VARIANT}_archive
    DEPENDS ${DOCKER_IMAGE}${DOCKER_VARIANT}_buildx
    COMMAND mkdir -p ${CMAKE_INSTALL_PREFIX}/docker
    COMMAND docker save ${DOCKER_REGISTRY}/${DOCKER_IMAGE}:${DOCKER_TAG}-${ARCH} --output ${CMAKE_CURRENT_BINARY_DIR}/${DOCKER_IMAGE}_${DOCKER_TAG}_${ARCH}.tar
)

add_custom_target(
    ${DOCKER_IMAGE}${DOCKER_VARIANT}_docker
    DEPENDS ${DOCKER_IMAGE}${DOCKER_VARIANT}_buildx
    DEPENDS ${DOCKER_IMAGE}${DOCKER_VARIANT}_archive
)
set_target_properties(
    ${DOCKER_IMAGE}${DOCKER_VARIANT}_docker PROPERTIES
    DOCKER_ARCHIVE ${CMAKE_INSTALL_PREFIX}/${PROJECT_NAME}/${DOCKER_IMAGE}_${DOCKER_TAG}_${ARCH}.tar
)

set_property(GLOBAL APPEND PROPERTY DOCKER_IMAGES ${DOCKER_IMAGE}${DOCKER_VARIANT}_docker)
