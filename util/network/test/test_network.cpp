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

#include "gtest/gtest.h"
#include "util/network/network.h"

TEST(util_network, ipv4_to_bits)
{
    const auto ipv4_valid = "192.168.99.21";
    const auto ipv4_invalid = "notanipaddress";

    const auto bits_valid = FLECS::ipv4_to_bits(ipv4_valid);
    const auto bits_invalid = FLECS::ipv4_to_bits(ipv4_invalid);

    ASSERT_EQ(ntohl(bits_valid.s_addr), in_addr_t{0xC0A86315});
    ASSERT_EQ(ntohl(bits_invalid.s_addr), in_addr_t{0x00000000});
}

TEST(util_network, ipv6_to_bits)
{
    const auto ipv6_valid_1 = "::1";
    const auto ipv6_valid_2 = "fe80::f003:edff:fe9d:4252";
    const auto ipv6_invalid = "notanipaddress";

    const auto bits_valid_1 = FLECS::ipv6_to_bits(ipv6_valid_1);
    const auto bits_valid_2 = FLECS::ipv6_to_bits(ipv6_valid_2);
    const auto bits_invalid = FLECS::ipv6_to_bits(ipv6_invalid);

    ASSERT_EQ(bits_valid_1.s6_addr[0], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[1], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[2], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[3], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[4], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[5], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[6], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[7], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[8], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[9], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[10], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[11], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[12], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[13], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[14], 0x00);
    ASSERT_EQ(bits_valid_1.s6_addr[15], 0x01);

    ASSERT_EQ(bits_valid_2.s6_addr[0], 0xFE);
    ASSERT_EQ(bits_valid_2.s6_addr[1], 0x80);
    ASSERT_EQ(bits_valid_2.s6_addr[2], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[3], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[4], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[5], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[6], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[7], 0x00);
    ASSERT_EQ(bits_valid_2.s6_addr[8], 0xF0);
    ASSERT_EQ(bits_valid_2.s6_addr[9], 0x03);
    ASSERT_EQ(bits_valid_2.s6_addr[10], 0xED);
    ASSERT_EQ(bits_valid_2.s6_addr[11], 0xFF);
    ASSERT_EQ(bits_valid_2.s6_addr[12], 0xFE);
    ASSERT_EQ(bits_valid_2.s6_addr[13], 0x9D);
    ASSERT_EQ(bits_valid_2.s6_addr[14], 0x42);
    ASSERT_EQ(bits_valid_2.s6_addr[15], 0x52);

    ASSERT_EQ(bits_invalid.s6_addr[0], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[1], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[2], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[3], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[4], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[5], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[6], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[7], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[8], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[9], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[10], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[11], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[12], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[13], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[14], 0x00);
    ASSERT_EQ(bits_invalid.s6_addr[15], 0x00);
}

TEST(util_network, ipv4_to_string)
{
    const auto ipv4_valid = 0xC0A86315;

    ASSERT_EQ(FLECS::ipv4_to_string(in_addr{.s_addr = ntohl(ipv4_valid)}), "192.168.99.21");
}

TEST(util_network, ipv6_to_string)
{
    const auto ipv6_valid_1 = 0xFE800000;
    const auto ipv6_valid_2 = 0x00000000;
    const auto ipv6_valid_3 = 0xF003EDFF;
    const auto ipv6_valid_4 = 0xFE9D4252;

    ASSERT_EQ(
        FLECS::ipv6_to_string(in6_addr{
            .__in6_u =
                {.__u6_addr32 = {htonl(ipv6_valid_1), htonl(ipv6_valid_2), htonl(ipv6_valid_3), htonl(ipv6_valid_4)}}}),
        "fe80::f003:edff:fe9d:4252");
}

TEST(util_network, subnet_mask_to_cidr_v4)
{
    const auto subnet_mask_1 = "255.255.252.0";
    const auto subnet_mask_2 = "255.255.0.0";
    const auto subnet_mask_invalid = "notasubnetmask";

    ASSERT_EQ(FLECS::subnet_mask_to_cidr_v4(subnet_mask_1), 22);
    ASSERT_EQ(FLECS::subnet_mask_to_cidr_v4(subnet_mask_2), 16);
    ASSERT_EQ(FLECS::subnet_mask_to_cidr_v4(subnet_mask_invalid), 0);
}

TEST(util_network, cidr_to_subnet_mask_v4)
{
    const auto cidr_1 = "192.168.178.0/24";
    const auto cidr_2 = "127.0.0.0/8";
    const auto cidr_invalid = "notacidrsubnet";

    ASSERT_EQ(FLECS::cidr_to_subnet_mask_v4(cidr_1), "255.255.255.0");
    ASSERT_EQ(FLECS::cidr_to_subnet_mask_v4(cidr_2), "255.0.0.0");
    ASSERT_EQ(FLECS::cidr_to_subnet_mask_v4(cidr_invalid), "");
}

TEST(util_network, ipv4_to_network)
{
    const auto ip_1 = "192.168.99.21";
    const auto subnet_mask_1 = "255.255.252.0";
    const auto ip_2 = "127.0.0.1";
    const auto subnet_mask_2 = "255.0.0.0";
    const auto ip_3 = "169.254.52.1";
    const auto subnet_mask_3 = "255.255.0.0";

    ASSERT_EQ(FLECS::ipv4_to_network(ip_1, subnet_mask_1), "192.168.96.0/22");
    ASSERT_EQ(FLECS::ipv4_to_network(ip_2, subnet_mask_2), "127.0.0.0/8");
    ASSERT_EQ(FLECS::ipv4_to_network(ip_3, subnet_mask_3), "169.254.0.0/16");
}
