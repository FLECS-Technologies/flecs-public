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

#include <string>

#include "util/string/string_utils.h"

TEST(string_utils, stringify_delim1)
{
    using std::string_literals::operator""s;

    const auto expected = std::string{"flecs\000test\000case"s};
    auto str1 = "flecs";
    auto str2 = "test";
    auto str3 = "case";

    const auto actual = flecs::stringify_delim('\0', str1, str2, str3);

    EXPECT_EQ(actual, expected);
}

TEST(string_utils, stringify_delim2)
{
    using std::string_literals::operator""s;

    const auto expected = std::string{"flecs-test-case"s};
    auto str1 = "flecs";
    auto str2 = "test";
    auto str3 = "case";

    const auto actual = flecs::stringify_delim('-', str1, str2, str3);

    EXPECT_EQ(actual, expected);
}

TEST(string_utils, stringify_delim3)
{
    using std::string_literals::operator""s;

    const auto expected = std::string{"flecs\000test-case\0003"s};
    auto str1 = std::string{"flecs"};
    auto str2 = "test-case";
    auto str3 = 3;

    const auto actual = flecs::stringify_delim('\0', str1, str2, str3);

    EXPECT_EQ(actual, expected);
}

TEST(string_utils, stringify_delim4)
{
    using std::string_literals::operator""s;

    const auto expected = std::string{"1,2,3,4,5"s};
    const auto v = std::vector<int>{1, 2, 3, 4, 5};

    const auto actual = flecs::stringify_delim(",", v);

    EXPECT_EQ(actual, expected);
}

TEST(string_utils, split1)
{
    const auto str = std::string{"flecs-test-case"};

    const auto actual = flecs::split(str, '-');

    ASSERT_EQ(actual.size(), 3);
    ASSERT_EQ(actual[0], "flecs");
    ASSERT_EQ(actual[1], "test");
    ASSERT_EQ(actual[2], "case");
}

TEST(string_utils, split2)
{
    const auto str = std::string{"flecs-test-case"};

    const auto actual = flecs::split(std::string_view{str}, '-');

    ASSERT_EQ(actual.size(), 3);
    ASSERT_EQ(actual[0], "flecs");
    ASSERT_EQ(actual[1], "test");
    ASSERT_EQ(actual[2], "case");
}

TEST(string_utils, split3)
{
    const auto str = std::string{"flecs-test-case"};

    const auto actual = flecs::split(str.c_str(), '-');

    ASSERT_EQ(actual.size(), 3);
    ASSERT_EQ(actual[0], "flecs");
    ASSERT_EQ(actual[1], "test");
    ASSERT_EQ(actual[2], "case");
}

TEST(string_utils, split_first1)
{
    const auto str = std::string{"flecs-test-case"};

    const auto [first, second] = flecs::split_first(str, '-');

    ASSERT_EQ(first, "flecs");
    ASSERT_EQ(second, "test-case");
}

TEST(string_utils, split_first2)
{
    const auto str = std::string{"flecs-"};

    const auto [first, second] = flecs::split_first(str, '-');

    ASSERT_EQ(first, "flecs");
    ASSERT_EQ(second, "");
}

TEST(string_utils, split_first3)
{
    const auto str = std::string{"flecs"};

    const auto [first, second] = flecs::split_first(str, '-');

    ASSERT_EQ(first, "flecs");
    ASSERT_EQ(second, "");
}

TEST(string_utils, split_first4)
{
    const auto str = std::string{"-"};

    const auto [first, second] = flecs::split_first(str, '-');

    ASSERT_EQ(first, "");
    ASSERT_EQ(second, "");
}

TEST(string_utils, ltrim)
{
    using std::operator""s;

    auto str = "\r\n\t    String with leading whitespaces    \t\r\n"s;

    ASSERT_EQ(flecs::ltrim(str), "String with leading whitespaces    \t\r\n");
}

TEST(string_utils, rtrim)
{
    using std::operator""s;

    auto str = "\r\n\t    String with trailing whitespaces    \t\r\n"s;

    ASSERT_EQ(flecs::rtrim(str), "\r\n\t    String with trailing whitespaces");
}

TEST(string_utils, trim)
{
    using std::operator""s;

    auto str = "\r\n\t    String with leading and trailing whitespaces    \t\r\n"s;

    ASSERT_EQ(flecs::trim(str), "String with leading and trailing whitespaces");
}
