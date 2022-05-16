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
};

TEST(module_marketplace, login)
{
    const auto user = "testuser";
    const auto token = "abcdef-1234-5678-XYZ";
    const auto out_expected = std::string{"{\"additionalInfo\":\"OK\"}"};

    auto mod = module_marketplace_test_t{};
    auto request = json_t{};
    request["user"] = user;
    request["token"] = token;

    auto response = json_t{};
    const auto res = mod.mp_login(request, response);

    ASSERT_EQ(res, FLECS::http_status_e::Ok);
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
    auto login_request = json_t{};
    login_request["user"] = user;
    login_request["token"] = token;

    auto logout_request = json_t{};
    logout_request["user"] = user;

    auto response = json_t{};
    (void)mod.mp_login(login_request, response);
    const auto res = mod.mp_logout(logout_request, response);

    ASSERT_EQ(res, FLECS::http_status_e::Ok);
    ASSERT_EQ(response.dump(), out_expected);
    ASSERT_TRUE(mod.user().empty());
    ASSERT_TRUE(mod.token().empty());
}
