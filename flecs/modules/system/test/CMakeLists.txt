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
    add_executable(modules.system.test
        test_system.cpp
    )

    set_target_properties(modules.system.test PROPERTIES OUTPUT_NAME test_daemon_modules_system)

    target_link_libraries(modules.system.test PRIVATE
        GTest::gtest
        GTest::gtest_main
        modules.factory
        modules.module_base
        modules.system
        magic::flecs_core_cxx_bridge
    )

    add_test(NAME modules.system.test COMMAND modules.system.test)
endif()
