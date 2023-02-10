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

if(BUILD_TESTING)
    project(FLECS.daemon.deployment.test)

    add_executable(FLECS.daemon.deployment.test
        test_deployment.cpp
    )

    set_target_properties(FLECS.daemon.deployment.test PROPERTIES OUTPUT_NAME test_daemon_deployment)

    target_include_directories(FLECS.daemon.deployment.test PRIVATE ${CMAKE_SOURCE_DIR}/daemon)

    target_link_libraries(FLECS.daemon.deployment.test PRIVATE
        FLECS.daemon.common.deployment
        FLECS.external.gmock
        external.gtest
        external.gtest_main
        FLECS.external.json
        FLECS.external.yaml-cpp
    )

    add_test(NAME FLECS.daemon.deployment.test COMMAND FLECS.daemon.deployment.test)
endif()
