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

#include "flecs/common/network/network.h"

TEST(network, default)
{
    const auto network = flecs::network_t{};

    ASSERT_FALSE(network.is_valid());
    ASSERT_TRUE(network.mac_address().empty());
    ASSERT_TRUE(network.name().empty());
    ASSERT_TRUE(network.parent().empty());
    ASSERT_EQ(network.type(), flecs::network_type_e::None);
}

TEST(network, custom)
{
    auto network = flecs::network_t{};

    network.type(flecs::network_type_from_string("ipvlan_l2"));
    network.mac_address("00:00:00:00:00:00");
    network.name("flecs-custom-ipvlan");
    network.parent("lo");

    ASSERT_TRUE(network.is_valid());
    ASSERT_EQ(network.mac_address(), "00:00:00:00:00:00");
    ASSERT_EQ(network.parent(), "lo");
    ASSERT_EQ(network.type(), flecs::network_type_e::IPVLAN_L2);
}
