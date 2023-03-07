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

#include "daemon/common/app/manifest/env_var/env_var.h"
#include "util/string/string_utils.h"

using std::operator""s;

TEST(env_var, valid)
{
    auto env_var1 = FLECS::env_var_t{"VALID_ENV_VAR1"};
    auto env_var2 = FLECS::env_var_t{"valid_env_var"};
    auto env_var3 = FLECS::env_var_t{"V1_"};

    ASSERT_TRUE(env_var1.is_valid());
    ASSERT_TRUE(env_var2.is_valid());
    ASSERT_TRUE(env_var3.is_valid());
}

TEST(env_var, invalid)
{
    auto env_var1 = FLECS::env_var_t{"_INVALID_ENV_VAR1"};
    auto env_var2 = FLECS::env_var_t{"INVALID ENV VAR"};
    auto env_var3 = FLECS::env_var_t{"1Invalid"};
    auto env_var4 = FLECS::env_var_t{"Invalid.Env.Var"};

    ASSERT_FALSE(env_var1.is_valid());
    ASSERT_FALSE(env_var2.is_valid());
    ASSERT_FALSE(env_var3.is_valid());
    ASSERT_FALSE(env_var4.is_valid());
}

TEST(env_var, mapped_valid)
{
    auto mapped_env_var1 = FLECS::mapped_env_var_t("VALID_ENV_VAR"s, "VALUE"s);
    auto mapped_env_var2 = FLECS::mapped_env_var_t("VALID_ENV_VAR"s, "VALUE"s);
    auto mapped_env_var3 = FLECS::mapped_env_var_t("VALID_ENV_VAR"s, "ANOTHER_VALUE"s);
    auto another_mapped_env_var1 = FLECS::mapped_env_var_t("ANOTHER_VALID_ENV_VAR"s, "VALUE"s);

    ASSERT_TRUE(mapped_env_var1.is_valid());
    ASSERT_EQ(FLECS::stringify(mapped_env_var1), "VALID_ENV_VAR:VALUE");
    ASSERT_EQ(mapped_env_var1, mapped_env_var2);
    ASSERT_EQ(mapped_env_var1, mapped_env_var3);
    ASSERT_NE(mapped_env_var1, another_mapped_env_var1);
}

TEST(env_var, mapped_invalid_1)
{
    auto mapped_env_var1 = FLECS::mapped_env_var_t{"_INVALID ENV_VAR"s, "val"s};

    ASSERT_FALSE(mapped_env_var1.is_valid());
    ASSERT_EQ(FLECS::stringify(mapped_env_var1), "");
}

TEST(env_var, mapped_invalid_2)
{
    auto mapped_env_var1 = FLECS::mapped_env_var_t{"_INVALID ENV_VAR"s};

    ASSERT_FALSE(mapped_env_var1.is_valid());
    ASSERT_EQ(FLECS::stringify(mapped_env_var1), "");
}

TEST(env_var, to_json)
{
    const auto mapped_env_var_1 = FLECS::mapped_env_var_t{"ENV_VAR"s, "VALUE"s};

    const auto json = FLECS::json_t(mapped_env_var_1);
    const auto json_expected = R"("ENV_VAR:VALUE")";

    ASSERT_TRUE(mapped_env_var_1.is_valid());
    ASSERT_EQ(json.dump(), json_expected);
}

TEST(env_var, from_json)
{
    const auto json = R"("ENV_VAR:VALUE")"_json;

    auto uut = FLECS::mapped_env_var_t{};
    from_json(json, uut);

    ASSERT_TRUE(uut.is_valid());
    ASSERT_EQ(uut.var(), "ENV_VAR");
    ASSERT_EQ(uut.value(), "VALUE");
}

TEST(env_var, to_string)
{
    const auto uut = FLECS::mapped_env_var_t{"ENV_VAR"s, "VALUE"s};

    ASSERT_EQ(to_string(uut), "ENV_VAR:VALUE");
}

TEST(env_var, sort)
{
    const auto uut_1 = FLECS::mapped_env_var_t{"ENV_VAR_1"s, "VALUE_1"};
    const auto uut_2 = FLECS::mapped_env_var_t{"ANOTHER_ENV_VAR"s, "A_VALUE"};

    ASSERT_TRUE(uut_1.is_valid());
    ASSERT_TRUE(uut_2.is_valid());

    ASSERT_LT(uut_2, uut_1);
    ASSERT_LE(uut_2, uut_1);
    ASSERT_NE(uut_2, uut_1);
    ASSERT_GE(uut_1, uut_2);
    ASSERT_GT(uut_1, uut_2);
}
