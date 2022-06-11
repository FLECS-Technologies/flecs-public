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

#include "network.h"

#include <regex>

#include "util/cxx20/string.h"

namespace FLECS {

network_t::network_t(std::string_view str)
    : _name{str}
    , _parent{}
    , _type{network_type_t::NONE}
{
    if (cxx20::contains(str, "-internal-"))
    {
        _type = network_type_t::INTERNAL;
    }
    else if (cxx20::contains(str, "-ipvlan-"))
    {
        _type = network_type_t::IPVLAN;

        const auto adapter_regex = std::regex{"ipvlan-(.+)$"};
        auto m = std::cmatch{};
        std::regex_search(str.data(), m, adapter_regex);
        if ((m.size() > 1) && (m[1].matched))
        {
            _parent = m[1];
        }
    }
    else if (cxx20::contains(str, "-macvlan-"))
    {
        _type = network_type_t::MACVLAN;

        const auto adapter_regex = std::regex{"macvlan-(.+)$"};
        auto m = std::cmatch{};
        std::regex_search(str.data(), m, adapter_regex);
        if ((m.size() > 1) && (m[1].matched))
        {
            _parent = m[1];
        }
    }
    else
    {
        _type = network_type_t::BRIDGE;
    }
}

auto network_t::name() const noexcept //
    -> const std::string&
{
    return _name;
}
auto network_t::parent() const noexcept //
    -> const std::string&
{
    return _parent;
}
auto network_t::type() const noexcept //
    -> network_type_t
{
    return _type;
}

auto network_t::is_valid() const noexcept //
    -> bool
{
    return !(_type == network_type_t::NONE);
}

auto to_json(json_t& j, const network_t& /*network*/) //
    -> void
{
    j = json_t{};
}

} // namespace FLECS
