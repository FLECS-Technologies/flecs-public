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

#pragma once

#include <map>
#include <string>
#include <string_view>

namespace FLECS {

enum class network_type_e {
    None,
    Internal,
    Bridge,
    MACVLAN,
    IPVLAN,
    Unknown,
};

static const auto strings = std::map<network_type_e, std::string>{
    {network_type_e::None, "none"},
    {network_type_e::Internal, "internal"},
    {network_type_e::Bridge, "bridge"},
    {network_type_e::MACVLAN, "macvlan"},
    {network_type_e::IPVLAN, "ipvlan"},
};

auto to_string_view(const network_type_e& network_type) //
    -> std::string_view;

auto to_string(const network_type_e& network_type) //
    -> std::string;

auto network_type_from_string(std::string_view str) //
    -> network_type_e;

} // namespace FLECS
