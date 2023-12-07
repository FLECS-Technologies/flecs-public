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

#include "network_type.h"

#include <algorithm>
#include <array>
#include <tuple>

namespace flecs {

static const auto mapping = std::array<std::tuple<network_type_e, std::string_view>, 5>{{
    {network_type_e::None, "none"},
    {network_type_e::Internal, "internal"},
    {network_type_e::Bridge, "bridge"},
    {network_type_e::MACVLAN, "macvlan"},
    {network_type_e::IPVLAN, "ipvlan"},
}};

auto to_string_view(const network_type_e& network_type) //
    -> std::string_view
{
    const auto it = std::find_if(
        mapping.cbegin(),
        mapping.cend(),
        [&network_type](const std::tuple<network_type_e, std::string_view>& elem) {
            return std::get<0>(elem) == network_type;
        });

    return it == mapping.cend() ? "unknown" : std::get<1>(*it);
}

auto to_string(const network_type_e& network_type) //
    -> std::string
{
    return std::string{to_string_view(network_type)};
}

auto network_type_from_string(std::string_view str) //
    -> network_type_e
{
    const auto it = std::find_if(
        mapping.cbegin(),
        mapping.cend(),
        [&str](const std::tuple<network_type_e, std::string_view>& elem) {
            return std::get<1>(elem) == str;
        });

    return it == mapping.cend() ? network_type_e::Unknown : std::get<0>(*it);
}

} // namespace flecs
