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
#include "marketplace/marketplace.h"

class module_marketplace_test_t : public FLECS::module_marketplace_t
{
public:
    module_marketplace_test_t() = default;

    auto login(std::string user, std::string token, FLECS::json_t& response)
    {
        return FLECS::module_marketplace_t::login(std::move(user), std::move(token), response);
    }

    auto logout(std::string_view user, FLECS::json_t& response)
    {
        return FLECS::module_marketplace_t::logout(std::move(user), response);
    }
};

TEST(module_marketplace, login)
{
    const auto user = "testuser";
    const auto token = "abcdef-1234-5678-XYZ";
    const auto out_expected = std::string{"{\"additionalInfo\":\"OK\"}"};

    auto mod = module_marketplace_test_t{};

    auto response = FLECS::json_t{};
    const auto res = mod.login(user, token, response);

    ASSERT_EQ(res.code, crow::status::OK);
    ASSERT_EQ(response.dump(), out_expected);
    ASSERT_EQ(mod.user(), user);
    ASSERT_EQ(mod.token(), token);
}

TEST(module_marketplace, logout)
{
    const auto user = "testuser";
    const auto token = "abcdef-1234-5678-XYZ";
    const auto out_expected = std::string{"{\"additionalInfo\":\"OK\"}"};

    auto mod = module_marketplace_test_t{};

    auto response = FLECS::json_t{};
    (void)mod.login(user, token, response);
    const auto res = mod.logout(user, response);

    ASSERT_EQ(res.code, crow::status::OK);
    ASSERT_EQ(response.dump(), out_expected);
    ASSERT_TRUE(mod.user().empty());
    ASSERT_TRUE(mod.token().empty());
}
