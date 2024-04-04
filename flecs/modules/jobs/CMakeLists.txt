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

project(modules.jobs.types)

add_library(${PROJECT_NAME} STATIC
    src/types/job_progress.cpp
    src/types/job_status.cpp
    src/types/job.cpp
    types/job_id.h
    types/job_progress.h
    types/job_status.h
    types/job.h
    types.h
)

flecs_add_module(
    MODULE_NAME jobs
    ADDITIONAL_HEADERS
        impl/jobs_impl.h
    ADDITIONAL_SOURCES
        src/impl/jobs_impl.cpp
    LIBS_PUBLIC
        modules.jobs.types
)

if(FLECS_BUILD_TESTS)
    add_library(daemon.modules.jobs.mocked_modules STATIC
        src/jobs.cpp
        src/impl/jobs_impl.cpp
        jobs.h
        impl/jobs_impl.h
    )

    target_link_libraries(
        daemon.modules.jobs.mocked_modules PUBLIC
        GTest::gmock
        daemon.modules.factory
        modules.jobs.types
        daemon.modules.module_base
    )

    target_compile_definitions(
        daemon.modules.jobs.mocked_modules PRIVATE
        -DFLECS_MOCK_MODULES
    )
endif()

add_subdirectory(test)
