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
    set(TEST_NAME modules.device.test)

    add_executable(${TEST_NAME}
        test_device.cpp
    )

    set_target_properties(${TEST_NAME} PROPERTIES OUTPUT_NAME test_device)

    target_link_libraries(${TEST_NAME} PRIVATE
        GTest::gmock_main
        modules.console.types
        modules.device.mocked_modules
        modules.factory
        util.signal_handler
        my::flecs-core-rs
    )

    add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})
endif()
