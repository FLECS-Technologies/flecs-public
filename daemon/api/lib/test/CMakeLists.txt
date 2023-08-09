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

if (BUILD_TESTING)
    project(daemon.api.lib.test)

    add_executable(daemon.api.lib.test
        test_api_lib.cpp
    )

    set_target_properties(daemon.api.lib.test PROPERTIES OUTPUT_NAME test_api_lib)

    target_link_libraries(daemon.api.lib.test PRIVATE
        external.gtest
        external.gtest_main
        daemon.lib
    )

    add_test(NAME daemon.api.lib.test COMMAND daemon.api.lib.test)
endif()
