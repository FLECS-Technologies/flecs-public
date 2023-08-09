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
    project(daemon.deployment.test)

    add_executable(daemon.deployment.test
        test_deployment.cpp
    )

    set_target_properties(daemon.deployment.test PROPERTIES OUTPUT_NAME test_daemon_deployment)

    target_include_directories(daemon.deployment.test PRIVATE ${CMAKE_SOURCE_DIR}/daemon)

    target_link_libraries(daemon.deployment.test PRIVATE
        daemon.common.deployment
        external.gmock
        external.gtest
        external.gtest_main
        external.yaml-cpp
    )

    add_test(NAME daemon.deployment.test COMMAND daemon.deployment.test)
endif()
