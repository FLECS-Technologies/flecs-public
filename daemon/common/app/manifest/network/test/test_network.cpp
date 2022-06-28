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

#include <gtest/gtest.h>

#include "daemon/common/app/manifest/network/network.h"

TEST(network, default)
{
    const auto network = FLECS::network_t{};

    ASSERT_FALSE(network.is_valid());
    ASSERT_TRUE(network.mac_address().empty());
    ASSERT_TRUE(network.name().empty());
    ASSERT_TRUE(network.parent().empty());
    ASSERT_EQ(network.type(), FLECS::network_type_t::NONE);
}

TEST(network, bridge)
{
    const auto network = FLECS::network_t{"flecs-bridge-custom"};

    ASSERT_TRUE(network.is_valid());
    ASSERT_TRUE(network.mac_address().empty());
    ASSERT_EQ(network.name(), "flecs-bridge-custom");
    ASSERT_TRUE(network.parent().empty());
    ASSERT_EQ(network.type(), FLECS::network_type_t::BRIDGE);
}

TEST(network, ipvlan)
{
    const auto network = FLECS::network_t{"flecs-ipvlan-lo"};

    ASSERT_TRUE(network.is_valid());
    // ASSERT_EQ(network.mac_address(), "00:00:00:00:00:00");
    ASSERT_EQ(network.name(), "flecs-ipvlan-lo");
    ASSERT_EQ(network.parent(), "lo");
    ASSERT_EQ(network.type(), FLECS::network_type_t::IPVLAN);
}

TEST(network, macvlan)
{
    const auto network = FLECS::network_t{"flecs-macvlan-lo"};

    ASSERT_TRUE(network.is_valid());
    // ASSERT_EQ(network.mac_address(), "00:00:00:00:00:00");
    ASSERT_EQ(network.name(), "flecs-macvlan-lo");
    ASSERT_EQ(network.parent(), "lo");
    ASSERT_EQ(network.type(), FLECS::network_type_t::MACVLAN);
}

TEST(network, internal)
{
    const auto network = FLECS::network_t{"flecs-internal-custom"};

    ASSERT_TRUE(network.is_valid());
    // ASSERT_EQ(network.mac_address(), "00:00:00:00:00:00");
    ASSERT_EQ(network.name(), "flecs-internal-custom");
    ASSERT_TRUE(network.parent().empty());
    ASSERT_EQ(network.type(), FLECS::network_type_t::INTERNAL);
}

TEST(network, custom)
{
    auto network = FLECS::network_t{};

    network.type(FLECS::network_type_from_string("ipvlan"));
    network.mac_address("00:00:00:00:00:00");
    network.name("flecs-custom-ipvlan");
    network.parent("lo");

    ASSERT_TRUE(network.is_valid());
    ASSERT_EQ(network.mac_address(), "00:00:00:00:00:00");
    ASSERT_EQ(network.parent(), "lo");
    ASSERT_EQ(network.type(), FLECS::network_type_t::IPVLAN);
}
