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
    project(FLECS.daemon.modules.marketplace.test)

    add_executable(FLECS.daemon.modules.marketplace.test
        test_marketplace.cpp
    )

    set_target_properties(FLECS.daemon.modules.marketplace.test PROPERTIES OUTPUT_NAME test_daemon_modules_marketplace)

    target_link_libraries(FLECS.daemon.modules.marketplace.test PRIVATE
        external.cpr
        FLECS.external.gtest
        FLECS.external.gtest_main
        FLECS.daemon.modules.factory
        FLECS.daemon.modules.module_base
        FLECS.daemon.modules.marketplace
    )

    add_test(NAME FLECS.daemon.modules.marketplace.test COMMAND FLECS.daemon.modules.marketplace.test)
endif()
