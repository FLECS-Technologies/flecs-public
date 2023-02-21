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

#include <cpr/cpr.h>

#include "gtest/gtest.h"
#include "marketplace/marketplace.h"

class module_marketplace_test_t : public FLECS::module_marketplace_t
{
public:
    module_marketplace_test_t() = default;

    auto do_init() //
        -> void override
    {
        return FLECS::module_marketplace_t::do_init();
    }
    auto do_deinit() //
        -> void override
    {
        return FLECS::module_marketplace_t::do_deinit();
    }

    auto login(std::string user, std::string token)
    {
        return FLECS::module_marketplace_t::login(std::move(user), std::move(token));
    }

    auto logout(std::string_view user)
    {
        return FLECS::module_marketplace_t::logout(std::move(user));
    }

    auto& user() const noexcept { return FLECS::module_marketplace_t::user(); }
    auto& token() const noexcept { return FLECS::module_marketplace_t::token(); }
};

class test_api_t
{
public:
    test_api_t()
        : _{}
    {}

    auto start() //
        -> void
    {
        _ = FLECS::flecs_api_t::instance()
                .app()
                .loglevel(crow::LogLevel::Critical)
                .bindaddr("127.0.0.1")
                .port(18951)
                .run_async();
        FLECS::flecs_api_t::instance().app().wait_for_server_start();
    }

    auto stop() //
        -> void
    {
        FLECS::flecs_api_t::instance().app().stop();
        _.wait();
    }

private:
    std::future<void> _;
};

static constexpr auto user = "testuser";
static constexpr auto token = "abcdef-1234-5678-XYZ";

static auto api = test_api_t{};
static auto uut = module_marketplace_test_t{};

TEST(marketplace, init)
{
    uut.do_init();
    api.start();
}

TEST(marketplace, login)
{
    using std::operator""s;

    const auto post_json = FLECS::json_t({{"user", user}, {"token", token}});
    const auto out_expected = R"({"additionalInfo":"OK"})"s;

    auto res = cpr::Post(
        cpr::Url{"http://127.0.0.1:18951/v2/marketplace/login"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}},
        cpr::Body{post_json.dump()});

    ASSERT_EQ(res.status_code, cpr::status::HTTP_OK);
    ASSERT_EQ(res.header.find("Content-Type")->second, "application/json");
    ASSERT_EQ(res.text, out_expected);
    ASSERT_EQ(uut.user(), user);
    ASSERT_EQ(uut.token(), token);
}

TEST(marketplace, logout)
{
    using std::operator""s;

    const auto post_json = FLECS::json_t({{"user", user}});
    const auto out_expected = R"({"additionalInfo":"OK"})"s;

    auto res = cpr::Post(
        cpr::Url{"http://127.0.0.1:18951/v2/marketplace/logout"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}},
        cpr::Body{post_json.dump()});

    ASSERT_EQ(res.status_code, cpr::status::HTTP_OK);
    ASSERT_EQ(res.header.find("Content-Type")->second, "application/json");
    ASSERT_EQ(res.text, out_expected);
    ASSERT_TRUE(uut.user().empty());
    ASSERT_TRUE(uut.token().empty());
}

TEST(marketplace, deinit)
{
    uut.do_deinit();
    api.stop();
}
