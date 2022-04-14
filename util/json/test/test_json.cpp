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
#include "util/json/json_parser.h"

TEST(util_json, cstr_success)
{
    const auto valid_json = "{\"key\":\"value\"}";
    const auto json = FLECS::parse_json(valid_json);

    ASSERT_TRUE(json.has_value());
    ASSERT_EQ(json.value()["key"], "value");
}

TEST(util_json, str_success)
{
    const auto valid_json = std::string{"{\"key\":\"value\"}"};
    const auto json = FLECS::parse_json(valid_json);

    ASSERT_TRUE(json.has_value());
    ASSERT_EQ(json.value()["key"], "value");
}

TEST(util_json, sv_success)
{
    const auto valid_json = std::string_view{"{\"key\":\"value\"}"};
    const auto json = FLECS::parse_json(valid_json);

    ASSERT_TRUE(json.has_value());
    ASSERT_EQ(json.value()["key"], "value");
}

TEST(util_json, cstr_fail)
{
    const auto invalid_json = "{\"key\",\"value\"}";

    const auto json = FLECS::parse_json(invalid_json);

    ASSERT_FALSE(json.has_value());
    ASSERT_ANY_THROW(json.value()["key"]);
}
