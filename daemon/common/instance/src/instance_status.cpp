// Copyright 2021-2022 FLECS Technologies GmbH
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
#include <map>
#include <tuple>

namespace FLECS {

auto to_char(const instance_status_e& instance_status) //
    -> char
{
    return static_cast<std::underlying_type_t<instance_status_e>>(instance_status);
}

auto to_string(const instance_status_e& instance_status) //
    -> std::string
{
    const auto strings = std::array<std::tuple<instance_status_e, std::string>, 6>{{
        {instance_status_e::CREATED, "created"},
        {instance_status_e::NOT_CREATED, "not created"},
        {instance_status_e::REQUESTED, "requested"},
        {instance_status_e::RESOURCES_READY, "resources ready"},
        {instance_status_e::RUNNING, "running"},
        {instance_status_e::STOPPED, "stopped"},
    }};

    const auto it = std::find_if(
        strings.cbegin(),
        strings.cend(),
        [&instance_status](const std::tuple<instance_status_e, std::string_view>& elem) {
            return std::get<0>(elem) == instance_status;
        });

    return it == strings.cend() ? "unknown" : std::get<1>(*it);
}

auto instance_status_from_string(std::string_view str) //
    -> instance_status_e
{
    const auto status = std::map<std::string_view, instance_status_e>{
        {"created", instance_status_e::CREATED},
        {"not created", instance_status_e::NOT_CREATED},
        {"requested", instance_status_e::REQUESTED},
        {"resources ready", instance_status_e::RESOURCES_READY},
        {"running", instance_status_e::RUNNING},
        {"stopped", instance_status_e::STOPPED},
    };

    return status.count(str) ? status.at(str) : instance_status_e::UNKNOWN;
}

} // namespace FLECS
