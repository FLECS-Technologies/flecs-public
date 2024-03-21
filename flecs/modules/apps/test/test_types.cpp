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

#include "flecs/modules/apps/types.h"

constexpr auto VALID_APP_NAME_1 = "tech.flecs.test-app";
constexpr auto VALID_APP_NAME_2 = "tech.flecs.test-app-2";
constexpr auto VALID_APP_VERSION_1 = "1.2.3.4-f1";
constexpr auto VALID_APP_VERSION_2 = "1.2.3.4-f2";

constexpr auto INVALID_APP_NAME = "a";
/// @todo constexpr auto INVALID_APP_VERSION = "=";

TEST(app_key, init)
{
    /* default constructor */
    {
        const auto uut = flecs::apps::key_t{};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.name(), "");
        ASSERT_EQ(uut.version(), "");
    }
    /* apps::key_t(std::string, std::string) */
    {
        const auto uut = flecs::apps::key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.name(), VALID_APP_NAME_1);
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
    /* apps::key_t(std::tuple<std::string, std::string>) */
    {
        const auto uut =
            flecs::apps::key_t{std::make_tuple(flecs::apps::name_t{VALID_APP_NAME_1}, VALID_APP_VERSION_1)};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.name(), VALID_APP_NAME_1);
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
    /* apps::key_t(app_name_t, std::string) */
    {
        const auto uut =
            flecs::apps::key_t{flecs::apps::name_t{std::string{INVALID_APP_NAME}}, VALID_APP_VERSION_1};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.name(), "");
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
}

TEST(app_key, sort)
{
    const auto uut_1 = flecs::apps::key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};
    const auto uut_2 = flecs::apps::key_t{VALID_APP_NAME_1, VALID_APP_VERSION_2};
    const auto uut_3 = flecs::apps::key_t{VALID_APP_NAME_2, VALID_APP_VERSION_2};

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
}

TEST(app_key, json)
{
    const auto json_expected = R"-({"name":"tech.flecs.test-app","version":"1.2.3.4-f1"})-";

    const auto uut_1 = flecs::apps::key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};
    const auto json = flecs::json_t(uut_1);

    ASSERT_EQ(json.dump(), json_expected);

    const auto uut_2 = json.get<flecs::apps::key_t>();
    ASSERT_EQ(uut_1, uut_2);
}

TEST(app_key, to_string)
{
    const auto expected = "tech.flecs.test-app (1.2.3.4-f1)";

    const auto uut = flecs::apps::key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};
    const auto str = to_string(uut);

    ASSERT_EQ(str, expected);
}

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
        const auto uut = flecs::apps::name_t{std::string{app_name}};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.value(), app_name);
    }
}

TEST(app_name, invalid)
{
    for (const auto& app_name : INVALID_APP_NAMES) {
        const auto uut = flecs::apps::name_t{std::string{app_name}};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.value(), "");
    }
}

TEST(app_name, sort)
{
    const auto app_1 = flecs::apps::name_t{"tech.flecs.app-1"};
    const auto app_2 = flecs::apps::name_t{"tech.flecs.app-2"};

    ASSERT_LT(app_1, app_2);
    ASSERT_LE(app_1, app_2);
    ASSERT_NE(app_1, app_2);
    ASSERT_EQ(app_1, app_1);
    ASSERT_GT(app_2, app_1);
    ASSERT_GE(app_2, app_1);
}

TEST(app_status, to_string)
{
    const auto values = std::array<flecs::apps::status_e, 10>{
        flecs::apps::status_e::NotInstalled,
        flecs::apps::status_e::ManifestDownloaded,
        flecs::apps::status_e::TokenAcquired,
        flecs::apps::status_e::ImageDownloaded,
        flecs::apps::status_e::Installed,
        flecs::apps::status_e::Removed,
        flecs::apps::status_e::Purged,
        flecs::apps::status_e::Orphaned,
        flecs::apps::status_e::Unknown,
        static_cast<flecs::apps::status_e>(-1),
    };

    const auto strings = std::array<std::string_view, 10>{
        "not installed",
        "manifest downloaded",
        "token acquired",
        "image downloaded",
        "installed",
        "removed",
        "purged",
        "orphaned",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(to_string(values[i]), strings[i]);
    }

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(to_string_view(values[i]), strings[i]);
    }

    /* skip last element as conversion is not bidirectional */
    for (size_t i = 0; i < values.size() - 1; ++i) {
        ASSERT_EQ(flecs::apps::status_from_string(strings[i]), values[i]);
    }
}
