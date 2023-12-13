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

#include <array>

#include "util/network/netif_type.h"

TEST(netif_type, to_string)
{
    const auto types = std::array<flecs::netif::type, 7>{
        flecs::netif::type::Wired,
        flecs::netif::type::Wireless,
        flecs::netif::type::Local,
        flecs::netif::type::Bridge,
        flecs::netif::type::Virtual,
        flecs::netif::type::Unknown,
        static_cast<flecs::netif::type>(-1),
    };

    const auto strings = std::array<std::string_view, 7>{
        "wired",
        "wireless",
        "local",
        "bridge",
        "virtual",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < types.size(); ++i) {
        ASSERT_EQ(to_string(types[i]), strings[i]);
    }

    for (size_t i = 0; i < types.size() - 1; ++i) {
        ASSERT_EQ(flecs::netif::from_string(strings[i]), types[i]);
    }
}

TEST(netif_type, from_adapter_name)
{
    const auto adapter_names = std::array<std::string_view, 11>{
        "lo",
        "lo127",
        "eth0",
        "enp4s1",
        "wlo1",
        "wlp0s20f3",
        "docker0",
        "br-35cb62",
        "vethde41@if62",
        "???",
        "custom-interface",
    };

    const auto types = std::array<flecs::netif::type, 11>{
        flecs::netif::type::Local,
        flecs::netif::type::Local,
        flecs::netif::type::Wired,
        flecs::netif::type::Wired,
        flecs::netif::type::Wireless,
        flecs::netif::type::Wireless,
        flecs::netif::type::Bridge,
        flecs::netif::type::Bridge,
        flecs::netif::type::Virtual,
        flecs::netif::type::Unknown,
        flecs::netif::type::Unknown,
    };

    for (size_t i = 0; i < adapter_names.size(); ++i) {
        ASSERT_EQ(flecs::netif::from_adapter_name(adapter_names[i]), types[i]);
    }
}
