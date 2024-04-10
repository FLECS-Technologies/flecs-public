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
    add_executable(modules.version.test
        test_version.cpp
    )

    set_target_properties(modules.version.test PROPERTIES OUTPUT_NAME test_daemon_modules_version)

    target_link_libraries(modules.version.test PRIVATE
        GTest::gtest
        GTest::gtest_main
        lib
        modules.factory
        modules.module_base
        modules.version
    )

    add_test(NAME modules.version.test COMMAND modules.version.test)
endif()
