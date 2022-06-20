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
    project(FLECS.util.network.test)

    add_executable(FLECS.test.util.network
        test_network.cpp
    )

    set_target_properties(FLECS.test.util.network PROPERTIES OUTPUT_NAME test_util_network)

    target_link_libraries(FLECS.test.util.network PRIVATE
        FLECS.external.gtest
        FLECS.external.gtest_main
        FLECS.util.network
    )

    add_test(NAME FLECS.test.util.network COMMAND FLECS.test.util.network)
endif()
