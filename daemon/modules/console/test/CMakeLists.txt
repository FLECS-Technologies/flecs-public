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
    project(daemon.modules.console.test)

    add_executable(daemon.modules.console.test
        test_console.cpp
        test_constants.cpp
        test_types.cpp
    )

    set_target_properties(daemon.modules.console.test PROPERTIES OUTPUT_NAME test_daemon_modules_console)

    target_link_libraries(daemon.modules.console.test PRIVATE
        cpr::cpr
        GTest::gtest
        GTest::gtest_main
        daemon.modules.factory
        daemon.modules.module_base
        daemon.modules.console
    )

    add_test(NAME daemon.modules.console.test COMMAND daemon.modules.console.test)
endif()
