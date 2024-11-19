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

add_library( modules.jobs.types STATIC
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
        util.signal_handler
)

add_subdirectory(test)
