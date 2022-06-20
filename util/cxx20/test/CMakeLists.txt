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
    project(FLECS.util.cxx20.test)

    add_executable(FLECS.util.cxx20.test
        test_cxx20.cpp
    )

    set_target_properties(FLECS.util.cxx20.test PROPERTIES OUTPUT_NAME test_util_string_utils)

    target_link_libraries(FLECS.util.cxx20.test PRIVATE
        FLECS.external.gtest
        FLECS.external.gtest_main
    )

    add_test(NAME FLECS.util.cxx20.test COMMAND FLECS.util.cxx20.test)
endif()
