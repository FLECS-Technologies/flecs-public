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

#include "flecs/util/network/ip_addr.h"

TEST(ip_addr, none)
{
    auto uut = flecs::ip_addr_t{};

    ASSERT_EQ(uut.type(), flecs::ip_addr_t::None);
    ASSERT_EQ(to_string(uut), "");
    ASSERT_ANY_THROW(uut.addr_v4());
    ASSERT_ANY_THROW(uut.addr_v6());
}

TEST(ip_addr, none_increment)
{
    auto uut_1 = flecs::ip_addr_t{};

    // prefix increment
    ++uut_1;
    ASSERT_EQ(uut_1.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_1.addr_v4());
    ASSERT_ANY_THROW(uut_1.addr_v6());

    // postfix increment
    auto uut_2 = uut_1++;
    ASSERT_EQ(uut_1.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_1.addr_v4());
    ASSERT_ANY_THROW(uut_1.addr_v6());
    ASSERT_EQ(uut_2.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_2.addr_v4());
    ASSERT_ANY_THROW(uut_2.addr_v6());

    // addition
    auto uut_3 = uut_2 + 2;
    ASSERT_EQ(uut_2.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_2.addr_v4());
    ASSERT_ANY_THROW(uut_2.addr_v6());
    ASSERT_EQ(uut_3.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_3.addr_v4());
    ASSERT_ANY_THROW(uut_3.addr_v6());

    uut_3 += 4;
    ASSERT_EQ(uut_3.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_3.addr_v4());
    ASSERT_ANY_THROW(uut_3.addr_v6());
}

TEST(ip_addr, none_decrement)
{
    auto uut_1 = flecs::ip_addr_t{};

    // prefix decrement
    --uut_1;
    ASSERT_EQ(uut_1.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_1.addr_v4());
    ASSERT_ANY_THROW(uut_1.addr_v6());

    // postfix decrement
    auto uut_2 = uut_1--;
    ASSERT_EQ(uut_1.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_1.addr_v4());
    ASSERT_ANY_THROW(uut_1.addr_v6());
    ASSERT_EQ(uut_2.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_2.addr_v4());
    ASSERT_ANY_THROW(uut_2.addr_v6());

    // subtraction
    auto uut_3 = uut_2 - 2;
    ASSERT_EQ(uut_2.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_2.addr_v4());
    ASSERT_ANY_THROW(uut_2.addr_v6());
    ASSERT_EQ(uut_3.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_3.addr_v4());
    ASSERT_ANY_THROW(uut_3.addr_v6());

    uut_3 -= 4;
    ASSERT_EQ(uut_3.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_3.addr_v4());
    ASSERT_ANY_THROW(uut_3.addr_v6());
}

TEST(ip_addr, none_compare)
{
    auto uut_1 = flecs::ip_addr_t{};
    auto uut_2 = flecs::ip_addr_t{};

    ASSERT_EQ(uut_1, uut_2);
    ASSERT_FALSE(uut_1 != uut_2);
    ASSERT_THROW(uut_1 < uut_2, std::runtime_error);
    ASSERT_THROW(uut_1 <= uut_2, std::runtime_error);
    ASSERT_THROW(uut_1 > uut_2, std::runtime_error);
    ASSERT_THROW(uut_1 >= uut_2, std::runtime_error);
}

TEST(ip_addr, ipv4)
{
    const auto ipv4_valid = "192.168.99.21";
    const auto ipv4_invalid = "notanipaddress";

    auto uut_valid_1 = flecs::ip_addr_t{ipv4_valid};
    auto uut_invalid = flecs::ip_addr_t{ipv4_invalid};

    ASSERT_EQ(uut_valid_1.type(), flecs::ip_addr_t::IPv4);
    ASSERT_EQ(uut_valid_1.addr_v4().s_addr, in_addr{.s_addr = htonl(0xC0A86315)}.s_addr);

    ASSERT_EQ(uut_invalid.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_invalid.addr_v4());
}

TEST(ip_addr, ipv4_increment)
{
    auto uut_1 = flecs::ip_addr_t{in_addr{.s_addr = htonl(0x7F0000FF)}};

    // prefix increment
    ++uut_1;
    ASSERT_EQ(uut_1.addr_v4().s_addr, htonl(0x7F000100));
    ASSERT_EQ(to_string(uut_1), "127.0.1.0");

    // postfix increment
    auto uut_2 = uut_1++;
    ASSERT_EQ(uut_1.addr_v4().s_addr, htonl(0x7F000101));
    ASSERT_EQ(to_string(uut_1), "127.0.1.1");
    ASSERT_EQ(uut_2.addr_v4().s_addr, htonl(0x7F000100));
    ASSERT_EQ(to_string(uut_2), "127.0.1.0");

    // addition
    auto uut_3 = uut_2 + 2;
    ASSERT_EQ(uut_3.addr_v4().s_addr, htonl(0x7F000102));
    ASSERT_EQ(to_string(uut_3), "127.0.1.2");
    uut_3 += 4;
    ASSERT_EQ(uut_3.addr_v4().s_addr, htonl(0x7F000106));
    ASSERT_EQ(to_string(uut_3), "127.0.1.6");
    uut_3 += -6;
    ASSERT_EQ(uut_3.addr_v4().s_addr, htonl(0x7F000100));
    ASSERT_EQ(to_string(uut_3), "127.0.1.0");
}

TEST(ip_addr, ipv4_decrement)
{
    auto uut_1 = flecs::ip_addr_t{in_addr{.s_addr = htonl(0x80000000)}};

    // prefix decrement
    --uut_1;
    ASSERT_EQ(uut_1.addr_v4().s_addr, htonl(0x7FFFFFFF));
    ASSERT_EQ(to_string(uut_1), "127.255.255.255");

    // postfix decrement
    auto uut_2 = uut_1--;
    ASSERT_EQ(uut_1.addr_v4().s_addr, htonl(0x7FFFFFFE));
    ASSERT_EQ(to_string(uut_1), "127.255.255.254");
    ASSERT_EQ(uut_2.addr_v4().s_addr, htonl(0x7FFFFFFF));
    ASSERT_EQ(to_string(uut_2), "127.255.255.255");

    // subtraction
    auto uut_3 = uut_2 - 2;
    ASSERT_EQ(uut_3.addr_v4().s_addr, htonl(0x7FFFFFFD));
    ASSERT_EQ(to_string(uut_3), "127.255.255.253");
    uut_3 -= 4;
    ASSERT_EQ(uut_3.addr_v4().s_addr, htonl(0x7FFFFFF9));
    ASSERT_EQ(to_string(uut_3), "127.255.255.249");
}

TEST(ip_addr, ipv4_compare)
{
    const auto uut_1 = flecs::ip_addr_t{in_addr{.s_addr = htonl(0x01020304)}};
    const auto uut_2 = flecs::ip_addr_t{in_addr{.s_addr = htonl(0x08090A0B)}};
    const auto uut_3 = flecs::ip_addr_t{htonl(0x08090A0B)};

    ASSERT_LT(uut_1, uut_2);
    ASSERT_LE(uut_1, uut_2);
    ASSERT_NE(uut_1, uut_2);
    ASSERT_GE(uut_2, uut_1);
    ASSERT_GT(uut_2, uut_1);
    ASSERT_EQ(uut_2, uut_3);

    const auto uut_empty = flecs::ip_addr_t{};
    ASSERT_ANY_THROW(uut_1 < uut_empty);
}

TEST(ip_addr, ipv6)
{
    const auto ipv6_valid_1 = "::1";
    const auto ipv6_valid_2 = "fe80::f003:edff:fe9d:4252";
    const auto ipv6_invalid = "notanipaddress";

    const auto uut_valid_1 = flecs::ip_addr_t{ipv6_valid_1};
    const auto uut_valid_2 = flecs::ip_addr_t{ipv6_valid_2};
    const auto uut_invalid = flecs::ip_addr_t{ipv6_invalid};

    const auto bits_valid_1 = uut_valid_1.addr_v6();
    ASSERT_EQ(uut_valid_1.type(), flecs::ip_addr_t::IPv6);
    ASSERT_EQ(bits_valid_1.s6_addr32[0], htonl(0x00000000));
    ASSERT_EQ(bits_valid_1.s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(bits_valid_1.s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(bits_valid_1.s6_addr32[3], htonl(0x00000001));

    const auto bits_valid_2 = uut_valid_2.addr_v6();
    ASSERT_EQ(uut_valid_2.type(), flecs::ip_addr_t::IPv6);
    ASSERT_EQ(bits_valid_2.s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(bits_valid_2.s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(bits_valid_2.s6_addr32[2], htonl(0xF003EDFF));
    ASSERT_EQ(bits_valid_2.s6_addr32[3], htonl(0xFE9D4252));

    ASSERT_EQ(uut_invalid.type(), flecs::ip_addr_t::None);
    ASSERT_ANY_THROW(uut_invalid.addr_v6());
}

TEST(ip_addr, ipv6_increment)
{
    auto uut_1 = flecs::ip_addr_t{"fe80::ffff:ffff:ffff:fffd"};

    // prefix increment
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0xFFFFFFFD));
    ++uut_1;
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0xFFFFFFFE));
    ASSERT_EQ(to_string(uut_1), "fe80::ffff:ffff:ffff:fffe");

    // postfix increment
    auto uut_2 = uut_1++;
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0xFFFFFFFF));
    ASSERT_EQ(to_string(uut_1), "fe80::ffff:ffff:ffff:ffff");
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[3], htonl(0xFFFFFFFE));
    ASSERT_EQ(to_string(uut_2), "fe80::ffff:ffff:ffff:fffe");

    // addition
    auto uut_3 = uut_2 + 2;
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[3], htonl(0xFFFFFFFE));
    ASSERT_EQ(to_string(uut_2), "fe80::ffff:ffff:ffff:fffe");
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0x00000000));
    ASSERT_EQ(to_string(uut_3), "fe80:0:0:1::");

    uut_3 += 4;
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0x00000004));
    ASSERT_EQ(to_string(uut_3), "fe80:0:0:1::4");

    uut_3 += -6;
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0xFFFFFFFE));
    ASSERT_EQ(to_string(uut_3), "fe80::ffff:ffff:ffff:fffe");
}

TEST(ip_addr, ipv6_decrement)
{
    auto uut_1 = flecs::ip_addr_t{"fe80:0:0:1::1"};

    // prefix decrement
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0x00000001));
    --uut_1;
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0x00000000));
    ASSERT_EQ(to_string(uut_1), "fe80:0:0:1::");

    // postfix decrement
    auto uut_2 = uut_1--;
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_1.addr_v6().s6_addr32[3], htonl(0xFFFFFFFF));
    ASSERT_EQ(to_string(uut_1), "fe80::ffff:ffff:ffff:ffff");
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[3], htonl(0x00000000));
    ASSERT_EQ(to_string(uut_2), "fe80:0:0:1::");

    // subtraction
    auto uut_3 = uut_2 - 2;
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_2.addr_v6().s6_addr32[3], htonl(0x00000000));
    ASSERT_EQ(to_string(uut_2), "fe80:0:0:1::");
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0xFFFFFFFE));
    ASSERT_EQ(to_string(uut_3), "fe80::ffff:ffff:ffff:fffe");

    uut_3 -= 4;
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0xFFFFFFFF));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0xFFFFFFFA));
    ASSERT_EQ(to_string(uut_3), "fe80::ffff:ffff:ffff:fffa");

    uut_3 -= -6;
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[0], htonl(0xFE800000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[1], htonl(0x00000001));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[2], htonl(0x00000000));
    ASSERT_EQ(uut_3.addr_v6().s6_addr32[3], htonl(0x00000000));
    ASSERT_EQ(to_string(uut_3), "fe80:0:0:1::");
}

TEST(ip_addr, ipv6_compare)
{
    auto ipv6_1 = in6_addr{};
    ipv6_1.s6_addr32[0] = htonl(0xFE800000);
    ipv6_1.s6_addr32[3] = htonl(0x00000100);

    auto ipv6_2 = in6_addr{};
    ipv6_2.s6_addr32[0] = htonl(0xFE800000);
    ipv6_2.s6_addr32[2] = htonl(0x00000001);
    ipv6_2.s6_addr32[3] = htonl(0x00000100);

    auto ipv6_3 = in6_addr{};
    ipv6_3.s6_addr32[0] = htonl(0xFE800000);
    ipv6_3.s6_addr32[1] = htonl(0x00000001);
    ipv6_3.s6_addr32[3] = htonl(0x00000100);

    const auto uut_1 = flecs::ip_addr_t{ipv6_1};
    const auto uut_2 = flecs::ip_addr_t{ipv6_2};
    const auto uut_3 = flecs::ip_addr_t{ipv6_3};

    ASSERT_LT(uut_1, uut_2);
    ASSERT_LE(uut_1, uut_2);
    ASSERT_NE(uut_1, uut_2);
    ASSERT_GE(uut_2, uut_1);
    ASSERT_GT(uut_2, uut_1);

    ASSERT_LT(uut_2, uut_3);
    ASSERT_LE(uut_2, uut_3);
    ASSERT_NE(uut_2, uut_3);
    ASSERT_GE(uut_3, uut_2);
    ASSERT_GT(uut_3, uut_2);

    ASSERT_NE(uut_1, flecs::ip_addr_t{});
}

TEST(ip_addr, assign)
{
    auto uut = flecs::ip_addr_t{};

    uut.addr(in_addr{});
    ASSERT_EQ(uut.type(), flecs::ip_addr_t::IPv4);
    ASSERT_NO_THROW(uut.addr_v4());
    ASSERT_ANY_THROW(uut.addr_v6());

    uut.addr(in6_addr{});
    ASSERT_EQ(uut.type(), flecs::ip_addr_t::IPv6);
    ASSERT_ANY_THROW(uut.addr_v4());
    ASSERT_NO_THROW(uut.addr_v6());

    uut.addr(0x7F000001);
    ASSERT_EQ(uut.type(), flecs::ip_addr_t::IPv4);
    ASSERT_NO_THROW(uut.addr_v4());
    ASSERT_ANY_THROW(uut.addr_v6());
}
