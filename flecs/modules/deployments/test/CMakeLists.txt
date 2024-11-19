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

if(FLECS_BUILD_TESTS)
    set(TEST_NAME modules.deployments.test)

    add_executable(${TEST_NAME}
        test_deployments.cpp
        test_docker.cpp
        test_types.cpp
    )

    set_target_properties(${TEST_NAME} PROPERTIES OUTPUT_NAME test_modules_deployments)

    target_link_libraries(${TEST_NAME} PRIVATE
        GTest::gmock
        GTest::gtest
        GTest::gtest_main
        modules.factory
        modules.module_base
        modules.deployments.mocked_modules
    )

    add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})
endif()
