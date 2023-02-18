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
    project(FLECS.util.container.test)

    add_executable(FLECS.util.container.test
        test_sorted_vector.cpp
    )

    set_target_properties(FLECS.util.container.test PROPERTIES OUTPUT_NAME test_util_container)

    target_link_libraries(FLECS.util.container.test PRIVATE
        external.gtest
        external.gtest_main
    )

    add_test(NAME FLECS.util.container.test COMMAND FLECS.util.container.test)
endif()
