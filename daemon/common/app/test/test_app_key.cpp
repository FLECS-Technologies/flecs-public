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

#include "daemon/common/app/app_key.h"
#include "gtest/gtest.h"

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
        const auto uut = FLECS::app_key_t{};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.name(), "");
        ASSERT_EQ(uut.version(), "");
    }
    /* app_key_t(std::string, std::string) */
    {
        const auto uut = FLECS::app_key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.name(), VALID_APP_NAME_1);
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
    /* app_key_t(std::tuple<std::string, std::string>) */
    {
        const auto uut = FLECS::app_key_t{
            std::make_tuple(FLECS::app_name_t{VALID_APP_NAME_1}, VALID_APP_VERSION_1)};

        ASSERT_TRUE(uut.is_valid());
        ASSERT_EQ(uut.name(), VALID_APP_NAME_1);
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
    /* app_key_t(app_name_t, std::string) */
    {
        const auto uut = FLECS::app_key_t{FLECS::app_name_t{std::string{INVALID_APP_NAME}}, VALID_APP_VERSION_1};

        ASSERT_FALSE(uut.is_valid());
        ASSERT_EQ(uut.name(), "");
        ASSERT_EQ(uut.version(), VALID_APP_VERSION_1);
    }
}

TEST(app_key, sort)
{
    const auto uut_1 = FLECS::app_key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};
    const auto uut_2 = FLECS::app_key_t{VALID_APP_NAME_1, VALID_APP_VERSION_2};
    const auto uut_3 = FLECS::app_key_t{VALID_APP_NAME_2, VALID_APP_VERSION_2};

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

    const auto uut_1 = FLECS::app_key_t{VALID_APP_NAME_1, VALID_APP_VERSION_1};
    const auto json = FLECS::json_t(uut_1);

    ASSERT_EQ(json.dump(), json_expected);

    const auto uut_2 = json.get<FLECS::app_key_t>();
    ASSERT_EQ(uut_1, uut_2);
}
