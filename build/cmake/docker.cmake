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

set(DOCKER_REGISTRY marketplace.flecs.tech:5001)
set(REGISTRY_USER $ENV{REGISTRY_USER})
set(REGISTRY_AUTH $ENV{REGISTRY_AUTH})
set(REGISTRY_PATH flecs)
set(DOCKER /usr/bin/docker)

if (NOT NDEBUG)
    set(DOCKER_TAG experimental)
else()
    set(DOCKER_TAG experimental)
endif()

add_custom_command(
    OUTPUT ${DOCKER_IMAGE}
    COMMAND ${DOCKER} login -u ${REGISTRY_USER} -p ${REGISTRY_AUTH} ${DOCKER_REGISTRY}
    COMMAND docker buildx build --push --build-arg MACHINE=${MACHINE} --build-arg ARCH=${ARCH} --build-arg VERSION=${VERSION} --platform ${DOCKER_ARCH} --tag ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:${ARCH}-${DOCKER_TAG} ${CMAKE_CURRENT_SOURCE_DIR}
    COMMAND docker manifest rm ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:${DOCKER_TAG} || true
    COMMAND docker manifest create ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:${DOCKER_TAG} ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:amd64-${DOCKER_TAG} ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:armhf-${DOCKER_TAG} ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:arm64-${DOCKER_TAG}  || true
    COMMAND docker manifest push ${DOCKER_REGISTRY}/${REGISTRY_PATH}/${DOCKER_IMAGE}:${DOCKER_TAG}
)

if (NOT TARGET ${DOCKER_IMAGE}_prepare)
    add_custom_target(${DOCKER_IMAGE}_prepare)
endif()

add_custom_target(
    ${DOCKER_IMAGE}_docker
    DEPENDS ${DOCKER_IMAGE}_prepare
    DEPENDS ${DOCKER_IMAGE}
)

list(APPEND DOCKER_IMAGES ${DOCKER_IMAGE}_docker)
set(DOCKER_IMAGES ${DOCKER_IMAGES} PARENT_SCOPE)
