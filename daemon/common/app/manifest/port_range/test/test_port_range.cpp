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

#include "daemon/common/app/manifest/port_range/port_range.h"

TEST(port_range, single_port)
{
    const auto str = "9000";

    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected =
        flecs::mapped_port_range_t{flecs::port_range_t{9000}, flecs::port_range_t{9000}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 9000);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9000);
    ASSERT_EQ(flecs::to_string(mapped_range), "9000:9000");
}

TEST(port_range, single_port_map)
{
    const auto str = "9000:9001";

    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected =
        flecs::mapped_port_range_t{flecs::port_range_t{9000}, flecs::port_range_t{9001}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 9000);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9001);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9001);
    ASSERT_EQ(flecs::to_string(mapped_range), "9000:9001");
}

TEST(port_range, single_port_map_random)
{
    const auto str = ":9001";
    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected =
        flecs::mapped_port_range_t{flecs::port_range_t{0, 0}, flecs::port_range_t{9001}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 0);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 0);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9001);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9001);
    ASSERT_EQ(flecs::to_string(mapped_range), ":9001");
}

TEST(port_range, port_range)
{
    const auto str = "9000-9005";
    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected = flecs::mapped_port_range_t{
        flecs::port_range_t{9000, 9005},
        flecs::port_range_t{9000, 9005}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 9005);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9005);
    ASSERT_EQ(flecs::to_string(mapped_range), "9000-9005:9000-9005");
}

TEST(port_range, port_range_map)
{
    const auto str = "9000-9005:9001-9006";
    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected = flecs::mapped_port_range_t{
        flecs::port_range_t{9000, 9005},
        flecs::port_range_t{9001, 9006}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 9000);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 9005);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9001);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9006);
    ASSERT_EQ(flecs::to_string(mapped_range), "9000-9005:9001-9006");
}

TEST(port_range, port_range_map_random)
{
    const auto str = ":9001-9006";
    const auto mapped_range = flecs::mapped_port_range_t{str};
    const auto mapped_range_expected =
        flecs::mapped_port_range_t{flecs::port_range_t{0, 0}, flecs::port_range_t{9001, 9006}};

    ASSERT_TRUE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range, mapped_range_expected);
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 0);
    ASSERT_EQ(mapped_range.host_port_range().end_port(), 0);
    ASSERT_EQ(mapped_range.container_port_range().start_port(), 9001);
    ASSERT_EQ(mapped_range.container_port_range().end_port(), 9006);
    ASSERT_EQ(flecs::to_string(mapped_range), ":9001-9006");
}

TEST(port_range, single_port_err)
{
    const auto str = "900a";
    const auto mapped_range = flecs::mapped_port_range_t{str};

    ASSERT_FALSE(mapped_range.is_valid());
    ASSERT_EQ(mapped_range.host_port_range().start_port(), 0);
}

TEST(port_range, single_port_map_err)
{
    const auto str1 = "9000:900a";
    const auto str2 = "900a:9000";
    const auto str3 = "900a:900a";

    const auto mapped_range1 = flecs::mapped_port_range_t{str1};
    const auto mapped_range2 = flecs::mapped_port_range_t{str2};
    const auto mapped_range3 = flecs::mapped_port_range_t{str3};

    ASSERT_FALSE(mapped_range1.is_valid());
    ASSERT_FALSE(mapped_range2.is_valid());
    ASSERT_FALSE(mapped_range3.is_valid());
}

TEST(port_range, single_port_map_random_err)
{
    const auto str = ":900a";

    const auto mapped_range = flecs::mapped_port_range_t{str};

    ASSERT_FALSE(mapped_range.is_valid());
}

TEST(port_range, port_range_err)
{
    const auto str1 = "900a-9006";
    const auto str2 = "9006-900a";
    const auto str3 = "🛫-🛬"; // airports are not allowed -.-
    const auto str4 = "∅";

    const auto mapped_range1 = flecs::mapped_port_range_t{str1};
    const auto mapped_range2 = flecs::mapped_port_range_t{str2};
    const auto mapped_range3 = flecs::mapped_port_range_t{str3};
    const auto mapped_range4 = flecs::mapped_port_range_t{str4};

    ASSERT_FALSE(mapped_range1.is_valid());
    ASSERT_FALSE(mapped_range2.is_valid());
    ASSERT_FALSE(mapped_range3.is_valid());
    ASSERT_FALSE(mapped_range4.is_valid());
}

TEST(port_range, port_range_invalid)
{
    const auto str1 = "9000-9005:9000-9001";
    const auto str2 = "9000:9000-9001";
    const auto str3 = "9000-9005:9000";

    const auto mapped_range1 = flecs::mapped_port_range_t{str1};
    const auto mapped_range2 = flecs::mapped_port_range_t{str2};
    const auto mapped_range3 = flecs::mapped_port_range_t{str3};

    ASSERT_FALSE(mapped_range1.is_valid());
    ASSERT_FALSE(mapped_range2.is_valid());
    ASSERT_FALSE(mapped_range3.is_valid());
}

TEST(port_range, to_json)
{
    const auto mapped_range_1 = flecs::mapped_port_range_t{"8000-8005:10000-10005"};

    const auto json = flecs::json_t(mapped_range_1);
    const auto json_expected = R"("8000-8005:10000-10005")";

    ASSERT_TRUE(mapped_range_1.is_valid());
    ASSERT_EQ(json.dump(), json_expected);
}

TEST(port_range, from_json)
{
    const auto json_string = R"("8000-8005:10000-10005")";
    auto json = flecs::parse_json(json_string);

    const auto mapped_range_1 = json.get<flecs::mapped_port_range_t>();

    ASSERT_TRUE(mapped_range_1.is_valid());
    ASSERT_EQ(mapped_range_1.host_port_range().start_port(), 8000);
    ASSERT_EQ(mapped_range_1.host_port_range().end_port(), 8005);
    ASSERT_EQ(mapped_range_1.container_port_range().start_port(), 10000);
    ASSERT_EQ(mapped_range_1.container_port_range().end_port(), 10005);
}
