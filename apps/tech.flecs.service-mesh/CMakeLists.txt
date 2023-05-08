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

project(apps.service-mesh)

# configure FLECS App
set(APP_NAME tech.flecs.service-mesh)
set(APP_VERSION ${CORE_VERSION})

include(flecs-app)

# configure Docker image
set(DOCKER_IMAGE ${APP_NAME})
set(DOCKER_TAG ${APP_VERSION})

install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/assets
    DESTINATION ${PROJECT_NAME}
    USE_SOURCE_PERMISSIONS
)

include(flecs-docker)
