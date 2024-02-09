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

if (FLECS_BUILD_TESTS)
    project(util.datetime.test)

    add_executable(util.datetime.test
        test_datetime.cpp
    )

    set_target_properties(util.datetime.test PROPERTIES OUTPUT_NAME test_util_datetime)

    target_link_libraries(util.datetime.test PRIVATE
        GTest::gtest
        GTest::gtest_main
        util.datetime
    )

    add_test(NAME util.datetime.test COMMAND util.datetime.test)
endif()
