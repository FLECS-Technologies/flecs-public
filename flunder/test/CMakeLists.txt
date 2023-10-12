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
    set(PROJECT_NAME flunder.test)
    project(${PROJECT_NAME})

    add_executable(${PROJECT_NAME}
        test_flunder.cpp
    )

    set_target_properties(${PROJECT_NAME} PROPERTIES OUTPUT_NAME test_flunder)

    target_link_libraries(${PROJECT_NAME} PRIVATE
        GTest::gtest
        GTest::gtest_main
        external.zenoh-c
        flunder
    )

    add_test(NAME flunder.test.init COMMAND bash -c "docker rm -f flecs-flunder-test && docker run -d --name flecs-flunder-test -p 7447:7447 -p 8000:8000 --env RUST_LOG=debug cr.flecs.tech/tech.flecs.service-mesh:2.4.0-dormouse-dev --adminspace-permissions=rw")
    add_test(NAME ${PROJECT_NAME} COMMAND ${PROJECT_NAME})
    set_tests_properties(${PROJECT_NAME} PROPERTIES DEPENDS flunder.test.init)
endif()