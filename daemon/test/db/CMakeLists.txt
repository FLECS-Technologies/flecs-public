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

if (BUILD_TESTING)
    project(FLECS.daemon.app_db.test)

    add_executable(FLECS.daemon.app_db.test
        test_app_db.cpp
        ${CMAKE_SOURCE_DIR}/daemon/src/db/app_db.cpp
    )

    set_target_properties(FLECS.daemon.app_db.test PROPERTIES OUTPUT_NAME test_daemon_app_db)

    target_include_directories(FLECS.daemon.app_db.test PRIVATE ${CMAKE_SOURCE_DIR}/daemon)

    target_compile_definitions(
        FLECS.daemon.app_db.test PRIVATE
        FLECS_APP_DB_PATH="${CMAKE_CURRENT_BINARY_DIR}/apps.db"
    )

    target_link_libraries(FLECS.daemon.app_db.test PRIVATE
        FLECS.external.gtest
        FLECS.external.gtest_main
        FLECS.util.sqlite3_ext
    )

    add_test(NAME FLECS.daemon.app_db.test COMMAND FLECS.daemon.app_db.test)
endif()
