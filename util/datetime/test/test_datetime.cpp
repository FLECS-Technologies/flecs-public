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

#include <regex>

#include "util/datetime/datetime.h"

TEST(datetime, strdate_time)
{
    // arbitrary timestamp, captured on testcase creation
    const auto timestamp_ns = 1645019968024874576;
    const auto timestamp_us = 1645019968024874;
    const auto timestamp_ms = 1645019968024;
    const auto timestamp_s = 1645019968;

    const auto expected_ns = "2022-02-16T13:59:28.024874576Z";
    const auto expected_us = "2022-02-16T13:59:28.024874Z";
    const auto expected_ms = "2022-02-16T13:59:28.024Z";
    const auto expected_s = "2022-02-16T13:59:28Z";

    const auto actual_ns = flecs::time_to_iso(timestamp_ns, flecs::precision_e::nanoseconds);
    const auto actual_us = flecs::time_to_iso(timestamp_us, flecs::precision_e::microseconds);
    const auto actual_ms = flecs::time_to_iso(timestamp_ms, flecs::precision_e::milliseconds);
    const auto actual_s = flecs::time_to_iso(timestamp_s, flecs::precision_e::seconds);

    EXPECT_EQ(actual_ns, expected_ns);
    EXPECT_EQ(actual_us, expected_us);
    EXPECT_EQ(actual_ms, expected_ms);
    EXPECT_EQ(actual_s, expected_s);
}

TEST(datetime, strdate_now)
{
    // regex matching any valid date string string between
    // 2000-01-01T00:00:00.000000000Z and
    // 2099-13-31T23:59:59.999999999Z
    // this at least catches errors such as misinterpreting the time unit
    using std::string_literals::operator""s;
    const auto regex_ns = std::regex{
        "^20[0-9]{2}-(?:1[0-2]|0[1-9])-(?:3[01]|[12][0-9]|0[1-9])T(?:2[0-3]|1[0-9]|0[0-9]):(?:[0-5][0-9]):(?:[0-5][0-9]).[0-9]{9}Z$"s};
    const auto regex_us = std::regex{
        "^20[0-9]{2}-(?:1[0-2]|0[1-9])-(?:3[01]|[12][0-9]|0[1-9])T(?:2[0-3]|1[0-9]|0[0-9]):(?:[0-5][0-9]):(?:[0-5][0-9]).[0-9]{6}Z$"s};
    const auto regex_ms = std::regex{
        "^20[0-9]{2}-(?:1[0-2]|0[1-9])-(?:3[01]|[12][0-9]|0[1-9])T(?:2[0-3]|1[0-9]|0[0-9]):(?:[0-5][0-9]):(?:[0-5][0-9]).[0-9]{3}Z$"s};
    const auto regex_s = std::regex{
        "^20[0-9]{2}-(?:1[0-2]|0[1-9])-(?:3[01]|[12][0-9]|0[1-9])T(?:2[0-3]|1[0-9]|0[0-9]):(?:[0-5][0-9]):(?:[0-5][0-9])Z$"s};

    const auto actual_ns = flecs::time_to_iso(flecs::precision_e::nanoseconds);
    const auto actual_us = flecs::time_to_iso(flecs::precision_e::microseconds);
    const auto actual_ms = flecs::time_to_iso(flecs::precision_e::milliseconds);
    const auto actual_s = flecs::time_to_iso(flecs::precision_e::seconds);

    ASSERT_TRUE(std::regex_search(actual_ns, regex_ns));
    ASSERT_TRUE(std::regex_search(actual_us, regex_us));
    ASSERT_TRUE(std::regex_search(actual_ms, regex_ms));
    ASSERT_TRUE(std::regex_search(actual_s, regex_s));

    ASSERT_FALSE(std::regex_search(actual_ns, regex_us));
    ASSERT_FALSE(std::regex_search(actual_ns, regex_ms));
    ASSERT_FALSE(std::regex_search(actual_ns, regex_s));
}

TEST(datetime, unix)
{
    const auto now_ns = flecs::unix_time(flecs::precision_e::nanoseconds);
    const auto now_us = flecs::unix_time(flecs::precision_e::microseconds);
    const auto now_ms = flecs::unix_time(flecs::precision_e::milliseconds);
    const auto now_s = flecs::unix_time(flecs::precision_e::seconds);

    /* length assertions hold true  until Nov. 2286 */
    ASSERT_EQ(now_ns.length(), 19);
    ASSERT_EQ(now_us.length(), 16);
    ASSERT_EQ(now_ms.length(), 13);
    ASSERT_EQ(now_s.length(), 10);

    /* value assertions hold true from Apr. 2022... */
    ASSERT_GT(std::stoull(now_ns), 1'650'000'000'000'000'000);
    ASSERT_GT(std::stoull(now_us), 1'650'000'000'000'000);
    ASSERT_GT(std::stoull(now_ms), 1'650'000'000'000);
    ASSERT_GT(std::stoull(now_s), 1'650'000'000);

    /* ... to May 2033 */
    ASSERT_LT(std::stoull(now_ns), 2'000'000'000'000'000'000);
    ASSERT_LT(std::stoull(now_us), 2'000'000'000'000'000);
    ASSERT_LT(std::stoull(now_ms), 2'000'000'000'000);
    ASSERT_LT(std::stoull(now_s), 2'000'000'000);
}
