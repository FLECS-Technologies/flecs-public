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

#include "job_status.h"

#include <algorithm>
#include <array>
#include <tuple>

namespace flecs {

auto to_string(job_status_e job_status) //
    -> std::string_view
{
    constexpr auto strings = std::array<std::tuple<job_status_e, std::string_view>, 6>{{
        {job_status_e::Pending, "pending"},
        {job_status_e::Queued, "queued"},
        {job_status_e::Running, "running"},
        {job_status_e::Cancelled, "cancelled"},
        {job_status_e::Successful, "successful"},
        {job_status_e::Failed, "failed"},
    }};

    const auto it = std::find_if(
        strings.cbegin(),
        strings.cend(),
        [&job_status](const std::tuple<job_status_e, std::string_view>& elem) {
            return std::get<0>(elem) == job_status;
        });

    return it == strings.cend() ? "unknown" : std::get<1>(*it);
}

} // namespace flecs
