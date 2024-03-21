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

#include "flecs/util/network/network.h"

TEST(util_network, subnet_mask_to_cidr_v4)
{
    const auto subnet_mask_1 = "255.255.252.0";
    const auto subnet_mask_2 = "255.255.0.0";
    const auto subnet_mask_invalid = "notasubnetmask";

    ASSERT_EQ(flecs::subnet_mask_to_cidr_v4(subnet_mask_1), 22);
    ASSERT_EQ(flecs::subnet_mask_to_cidr_v4(subnet_mask_2), 16);
    ASSERT_EQ(flecs::subnet_mask_to_cidr_v4(subnet_mask_invalid), 0);
}

TEST(util_network, cidr_to_subnet_mask_v4)
{
    const auto cidr_1 = "192.168.178.0/24";
    const auto cidr_2 = "127.0.0.0/8";
    const auto cidr_invalid = "notacidrsubnet";

    ASSERT_EQ(flecs::cidr_to_subnet_mask_v4(cidr_1), "255.255.255.0");
    ASSERT_EQ(flecs::cidr_to_subnet_mask_v4(cidr_2), "255.0.0.0");
    ASSERT_EQ(flecs::cidr_to_subnet_mask_v4(cidr_invalid), "");
}

TEST(util_network, ipv4_to_network)
{
    const auto ip_1 = "192.168.99.21";
    const auto subnet_mask_1 = "255.255.252.0";
    const auto ip_2 = "127.0.0.1";
    const auto subnet_mask_2 = "255.0.0.0";
    const auto ip_3 = "169.254.52.1";
    const auto subnet_mask_3 = "255.255.0.0";

    ASSERT_EQ(flecs::ipv4_to_network(ip_1, subnet_mask_1), "192.168.96.0/22");
    ASSERT_EQ(flecs::ipv4_to_network(ip_2, subnet_mask_2), "127.0.0.0/8");
    ASSERT_EQ(flecs::ipv4_to_network(ip_3, subnet_mask_3), "169.254.0.0/16");
}
