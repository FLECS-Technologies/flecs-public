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
    add_executable(test.util.network
        test_ip_addr.cpp
        test_netif_type.cpp
        test_network.cpp
    )

    set_target_properties(test.util.network PROPERTIES OUTPUT_NAME test_util_network)

    target_link_libraries(test.util.network PRIVATE
        GTest::gtest
        GTest::gtest_main
        util.network
    )

    add_test(NAME test.util.network COMMAND test.util.network)
endif()
