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
    project(daemon.app.startup_option.test)

    add_executable(daemon.app.startup_option.test
        test_startup_option.cpp
    )

    set_target_properties(daemon.app.startup_option.test PROPERTIES OUTPUT_NAME test_daemon_app_startup_option)

    target_link_libraries(daemon.app.startup_option.test PRIVATE
        GTest::gtest
        GTest::gtest_main
        daemon.common.app.manifest.startup_option
    )

    add_test(NAME daemon.app.startup_option.test COMMAND daemon.app.startup_option.test)
endif()
