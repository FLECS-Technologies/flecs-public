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

#include "daemon/common/app/app_name.h"

constexpr auto VALID_APP_NAMES = std::array<std::string_view, 4>{{
    "tech.flecs.a",
    "tech.flecs.app-1",
    "tech.flecs.app-1.extension",
    "tech.flecs.perfectly-valid-app-name-although-it-is-riiiight-at-the-edge-of-"
    "being-rejected-due-to-length-limitation-of-128-chars",
}};

constexpr auto INVALID_APP_NAMES = std::array<std::string_view, 11>{{
    "Tech.flecs.app-1",  // starts with forbidden character
    "2tech.flecs.app-1", // starts with forbidden character
    "-tech.flecs.app-1", // starts with forbidden character
    "tech.flecs-.app-1", // company ends with forbidden character
    "tech.flecs.app-1-", // ends with forbidden character
    "tech.flecs-app.-",  // ends with forbidden character
    "tech.flecs-app.",   // ends with forbidden character
    "tech.flecs-app",    // missing product name
    "tech.flecs.perfectly-valid-app-name-but-in-the-end-just-waaaaaaaaaaaaaaaaaay-too-long-so-it-"
    "is-"
    "rejected-due-to-length-limitation", // exceeds character limit
    "com2.flecs.app-1",                  // forbidden character in TLD
    "tech.flecs.app_1",                  // forbidden character in product name
}};

TEST(app_name, valid)
{
    for (const auto& app_name : VALID_APP_NAMES) {
        const auto uut = flecs::app_name_t{std::string{app_name}};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.value(), app_name);
    }
}

TEST(app_name, invalid)
{
    for (const auto& app_name : INVALID_APP_NAMES) {
        const auto uut = flecs::app_name_t{std::string{app_name}};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.value(), "");
    }
}

TEST(app_name, sort)
{
    const auto app_1 = flecs::app_name_t{"tech.flecs.app-1"};
    const auto app_2 = flecs::app_name_t{"tech.flecs.app-2"};

    ASSERT_LT(app_1, app_2);
    ASSERT_LE(app_1, app_2);
    ASSERT_NE(app_1, app_2);
    ASSERT_EQ(app_1, app_1);
    ASSERT_GT(app_2, app_1);
    ASSERT_GE(app_2, app_1);
}
