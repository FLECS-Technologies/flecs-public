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

#include "instance_status.h"

#include <algorithm>
#include <array>
#include <tuple>

namespace flecs {

auto to_string_view(const instance_status_e& instance_status) //
    -> std::string_view
{
    const auto strings = std::array<std::tuple<instance_status_e, std::string_view>, 7>{{
        {instance_status_e::Created, "created"},
        {instance_status_e::NotCreated, "not created"},
        {instance_status_e::Orphaned, "orphaned"},
        {instance_status_e::Requested, "requested"},
        {instance_status_e::ResourcesReady, "resources ready"},
        {instance_status_e::Running, "running"},
        {instance_status_e::Stopped, "stopped"},
    }};

    const auto it = std::find_if(
        strings.cbegin(),
        strings.cend(),
        [&instance_status](const std::tuple<instance_status_e, std::string_view>& elem) {
            return std::get<0>(elem) == instance_status;
        });

    return it == strings.cend() ? "unknown" : std::get<1>(*it);
}

auto to_string(const instance_status_e& instance_status) //
    -> std::string
{
    return std::string{to_string_view(instance_status)};
}

auto instance_status_from_string(std::string_view str) //
    -> instance_status_e
{
    const auto status = std::array<std::tuple<std::string_view, instance_status_e>, 7>{{
        {"created", instance_status_e::Created},
        {"not created", instance_status_e::NotCreated},
        {"orphaned", instance_status_e::Orphaned},
        {"requested", instance_status_e::Requested},
        {"resources ready", instance_status_e::ResourcesReady},
        {"running", instance_status_e::Running},
        {"stopped", instance_status_e::Stopped},
    }};

    const auto it = std::find_if(
        status.cbegin(),
        status.cend(),
        [&str](const std::tuple<std::string_view, instance_status_e>& elem) {
            return std::get<0>(elem) == str;
        });

    return it == status.cend() ? instance_status_e::Unknown : std::get<1>(*it);
}

} // namespace flecs
