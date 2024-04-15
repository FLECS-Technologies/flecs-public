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

#include "flecs/common/app/manifest/variable/variable.h"
#include "flecs/util/string/string_utils.h"

using std::operator""s;

TEST(env_var, valid)
{
    auto env_var1 = flecs::var_t::parse_env_var_name("VALID_ENV_VAR1");
    auto env_var2 = flecs::var_t::parse_env_var_name("valid_env_var");
    auto env_var3 = flecs::var_t::parse_env_var_name("V1_");
    auto env_var4 = flecs::var_t::parse_env_var_name("valid-env-var");
    auto env_var5 = flecs::var_t::parse_env_var_name("valid.env_var-2");

    ASSERT_TRUE(env_var1.has_value());
    ASSERT_TRUE(env_var1->is_valid());
    ASSERT_TRUE(env_var2.has_value());
    ASSERT_TRUE(env_var2->is_valid());
    ASSERT_TRUE(env_var3.has_value());
    ASSERT_TRUE(env_var3->is_valid());
    ASSERT_TRUE(env_var4.has_value());
    ASSERT_TRUE(env_var4->is_valid());
    ASSERT_TRUE(env_var5.has_value());
    ASSERT_TRUE(env_var5->is_valid());
}

TEST(env_var, invalid)
{
    auto env_var1 = flecs::var_t::parse_env_var_name("_INVALID_ENV_VAR1");
    auto env_var2 = flecs::var_t::parse_env_var_name("INVALID ENV VAR");
    auto env_var3 = flecs::var_t::parse_env_var_name("1Invalid");

    ASSERT_FALSE(env_var1.has_value());
    ASSERT_FALSE(env_var2.has_value());
    ASSERT_FALSE(env_var3.has_value());
}

TEST(env_var, mapped_valid)
{
    auto mapped_env_var1 = flecs::mapped_env_var_t::try_parse("VALID_ENV_VAR=VALUE"s);
    auto mapped_env_var2 = flecs::mapped_env_var_t::try_parse("VALID_ENV_VAR=VALUE"s);
    auto mapped_env_var3 = flecs::mapped_env_var_t::try_parse("VALID_ENV_VAR=ANOTHER_VALUE"s);
    auto mapped_env_var4 = flecs::mapped_env_var_t::try_parse("another.valid-env_var.2=some special! value?"s);

    ASSERT_TRUE(mapped_env_var1.has_value());
    ASSERT_TRUE(mapped_env_var2.has_value());
    ASSERT_TRUE(mapped_env_var3.has_value());
    ASSERT_TRUE(mapped_env_var4.has_value());
    ASSERT_EQ(flecs::stringify(mapped_env_var1.value()), "VALID_ENV_VAR=VALUE");
    ASSERT_EQ(mapped_env_var1.value(), mapped_env_var2.value());
    ASSERT_EQ(mapped_env_var1.value(), mapped_env_var3.value());
    ASSERT_NE(mapped_env_var1.value(), mapped_env_var4.value());
    ASSERT_EQ(flecs::stringify(mapped_env_var4.value()), "another.valid-env_var.2=some special! value?");
}

TEST(env_var, mapped_invalid_1)
{
    auto mapped_env_var1 = flecs::mapped_env_var_t::try_parse("_INVALID ENV_VAR=val"s);

    ASSERT_FALSE(mapped_env_var1.has_value());
}

TEST(env_var, mapped_invalid_2)
{
    auto mapped_env_var1 = flecs::mapped_env_var_t::try_parse("_INVALID ENV_VAR"s);

    ASSERT_FALSE(mapped_env_var1.has_value());
}

TEST(env_var, to_json)
{
    const auto mapped_env_var_1 = flecs::mapped_env_var_t::try_parse("ENV_VAR=VALUE"s);

    ASSERT_TRUE(mapped_env_var_1.has_value());
    const auto json = flecs::json_t(mapped_env_var_1.value());
    const auto json_expected = R"("ENV_VAR=VALUE")";

    ASSERT_EQ(json.dump(), json_expected);
}

TEST(env_var, from_json)
{
    auto uut = flecs::mapped_env_var_t{};

    const auto json_1 = R"("ENV_VAR:VALUE")"_json;
    from_json(json_1, uut);

    ASSERT_TRUE(uut.is_valid());
    ASSERT_EQ(uut.var(), "ENV_VAR");
    ASSERT_EQ(uut.value(), "VALUE");

    const auto json_2 = R"("ENV_VAR=VALUE")"_json;
    from_json(json_2, uut);

    ASSERT_TRUE(uut.is_valid());
    ASSERT_EQ(uut.var(), "ENV_VAR");
    ASSERT_EQ(uut.value(), "VALUE");

    const auto json_3 = R"("PATH:/bin:/usr/bin:/sbin:/usr/sbin")"_json;
    from_json(json_3, uut);

    ASSERT_TRUE(uut.is_valid());
    ASSERT_EQ(uut.var(), "PATH");
    ASSERT_EQ(uut.value(), "/bin:/usr/bin:/sbin:/usr/sbin");

    const auto json_4 = R"("PATH=/bin:/usr/bin:/sbin:/usr/sbin")"_json;
    from_json(json_4, uut);

    ASSERT_TRUE(uut.is_valid());
    ASSERT_EQ(uut.var(), "PATH");
    ASSERT_EQ(uut.value(), "/bin:/usr/bin:/sbin:/usr/sbin");
}

TEST(env_var, to_string)
{
    const auto uut = flecs::mapped_env_var_t::try_parse("ENV_VAR=VALUE"s);
    ASSERT_TRUE(uut.has_value());
    ASSERT_EQ(to_string(uut.value()), "ENV_VAR=VALUE");
}

TEST(env_var, sort)
{
    const auto uut_1 = flecs::mapped_env_var_t::try_parse("ENV_VAR_1=VALUE_1");
    const auto uut_2 = flecs::mapped_env_var_t::try_parse("ANOTHER_ENV_VAR=A_VALUE");

    ASSERT_TRUE(uut_1.has_value());
    ASSERT_TRUE(uut_2.has_value());

    ASSERT_LT(uut_2.value(), uut_1.value());
    ASSERT_LE(uut_2.value(), uut_1.value());
    ASSERT_NE(uut_2.value(), uut_1.value());
    ASSERT_GE(uut_1.value(), uut_2.value());
    ASSERT_GT(uut_1.value(), uut_2.value());
}
