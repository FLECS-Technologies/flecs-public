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
    project(FLECS.util.string.utils.test)

    add_executable(FLECS.util.string.utils.test
        test_comparator.cpp
        test_format.cpp
        test_string_utils.cpp
    )

    set_target_properties(FLECS.util.string.utils.test PROPERTIES OUTPUT_NAME test_util_string_utils)

    target_link_libraries(FLECS.util.string.utils.test PRIVATE
        FLECS.external.gtest
        FLECS.external.gtest_main
    )

    add_test(NAME FLECS.util.string.utils.test COMMAND FLECS.util.string.utils.test)
endif()
