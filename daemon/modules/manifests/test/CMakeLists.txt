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

if(BUILD_TESTING)
    set(PROJECT_NAME daemon.modules.manifests.test)
    project(${PROJECT_NAME})

    add_executable(${PROJECT_NAME}
        test_manifests.cpp
    )

    set_target_properties(${PROJECT_NAME} PROPERTIES OUTPUT_NAME test_manifests)

    target_link_libraries(${PROJECT_NAME} PRIVATE
        FLECS.external.gtest
        FLECS.external.gtest_main
        FLECS.daemon.common.app.manifest
        FLECS.daemon.modules.manifests
    )

    add_test(NAME ${PROJECT_NAME} COMMAND ${PROJECT_NAME})
endif()
