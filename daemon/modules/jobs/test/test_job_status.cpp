// Copyright 2021-2023 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <gtest/gtest.h>

#include "daemon/modules/jobs/job_status.h"

TEST(job_status, to_string)
{
    const auto status = std::array<FLECS::job_status_e, 7>{{
        FLECS::job_status_e::Pending,
        FLECS::job_status_e::Queued,
        FLECS::job_status_e::Running,
        FLECS::job_status_e::Cancelled,
        FLECS::job_status_e::Successful,
        FLECS::job_status_e::Failed,
        static_cast<FLECS::job_status_e>(-1),
    }};

    const auto expected = std::array<std::string_view, 7>{
        {"pending", "queued", "running", "cancelled", "successful", "failed", "unknown"}};

    for (size_t i = 0; i < status.size(); ++i) {
        ASSERT_EQ(to_string(status[i]), expected[i]);
    }
}