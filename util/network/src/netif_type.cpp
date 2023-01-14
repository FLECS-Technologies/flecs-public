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

#include "netif_type.h"

#include <algorithm>
#include <array>
#include <tuple>

#include "util/cxx20/string.h"

namespace FLECS {
namespace netif {

auto to_string_view(type netif_type) //
    -> std::string_view
{
    constexpr auto strings = std::array<std::tuple<type, std::string_view>, 5>{{
        {type::Wired, "wired"},
        {type::Wireless, "wireless"},
        {type::Local, "local"},
        {type::Bridge, "bridge"},
        {type::Virtual, "virtual"},
    }};

    const auto it = std::find_if(
        strings.cbegin(),
        strings.cend(),
        [&netif_type](const std::tuple<type, std::string_view>& elem) {
            return std::get<0>(elem) == netif_type;
        });

    return it == strings.cend() ? "unknown" : std::get<1>(*it);
}

auto to_string(type netif_type) //
    -> std::string
{
    return std::string{to_string_view(netif_type)};
}

auto from_string(std::string_view str) //
    -> type
{
    constexpr auto types = std::array<std::tuple<std::string_view, type>, 5>{{
        {"wired", type::Wired},
        {"wireless", type::Wireless},
        {"local", type::Local},
        {"bridge", type::Bridge},
        {"virtual", type::Virtual},
    }};

    const auto it = std::find_if(
        types.cbegin(),
        types.cend(),
        [&str](const std::tuple<std::string_view, type>& elem) {
            return std::get<0>(elem) == str;
        });

    return it == types.cend() ? type::Unknown : std::get<1>(*it);
}

auto from_adapter_name(std::string_view str) //
    -> type
{
    if ((cxx20::starts_with(str, "en") || (cxx20::starts_with(str, "eth")))) {
        return type::Wired;
    } else if ((cxx20::starts_with(str, "wl"))) {
        return type::Wireless;
    } else if ((cxx20::starts_with(str, "lo"))) {
        return type::Local;
    } else if ((cxx20::starts_with(str, "veth"))) {
        return type::Virtual;
    } else if ((cxx20::starts_with(str, "br") || (cxx20::starts_with(str, "docker")))) {
        return type::Bridge;
    }

    return type::Unknown;
}

} // namespace netif
} // namespace FLECS
