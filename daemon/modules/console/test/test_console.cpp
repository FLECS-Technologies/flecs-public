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

#include "console/console.h"
#include "gtest/gtest.h"
#include "test_constants.h"

class module_console_test_t : public flecs::module::console_t
{
public:
    module_console_test_t() = default;

    auto do_init() //
        -> void override
    {
        return flecs::module::console_t::do_init();
    }
    auto do_deinit() //
        -> void override
    {
        return flecs::module::console_t::do_deinit();
    }
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
        _ = flecs::flecs_api_t::instance()
                .app()
                .loglevel(crow::LogLevel::Critical)
                .bindaddr("127.0.0.1")
                .port(18951)
                .run_async();
        flecs::flecs_api_t::instance().app().wait_for_server_start();
    }

    auto stop() //
        -> void
    {
        flecs::flecs_api_t::instance().app().stop();
        _.wait();
    }

private:
    std::future<void> _;
};

static constexpr auto user = "testuser";
static constexpr auto token = "abcdef-1234-5678-XYZ";

static auto api = test_api_t{};
static auto uut = module_console_test_t{};

TEST(console, init)
{
    uut.init();
    api.start();
}

TEST(console, base_url)
{
    const auto url = uut.base_url();

    ASSERT_EQ(url, "https://console-dev.flecs.tech");
}

TEST(console, store_delete_authentication)
{
    using std::operator""s;

    ASSERT_EQ(uut.authentication().user().id(), 0);
    ASSERT_EQ(uut.authentication().user().user_email(), std::string{});
    ASSERT_EQ(uut.authentication().user().user_login(), std::string{});
    ASSERT_EQ(uut.authentication().user().display_name(), std::string{});
    ASSERT_EQ(uut.authentication().jwt().token(), std::string{});
    ASSERT_EQ(uut.authentication().jwt().token_expires(), 0);
    ASSERT_EQ(uut.authentication().feature_flags().is_vendor(), false);
    ASSERT_EQ(uut.authentication().feature_flags().is_white_labeled(), false);

    auto res = cpr::Put(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}},
        cpr::Body{auth_response_json.dump()});
    ASSERT_EQ(res.status_code, cpr::status::HTTP_NO_CONTENT);

    ASSERT_EQ(uut.authentication().user().id(), 123);
    ASSERT_EQ(uut.authentication().user().user_email(), "user@flecs.tech");
    ASSERT_EQ(uut.authentication().user().user_login(), "user");
    ASSERT_EQ(uut.authentication().user().display_name(), "Some FLECS user");
    ASSERT_EQ(uut.authentication().jwt().token(), "eyJ0eXAiO...");
    ASSERT_EQ(uut.authentication().jwt().token_expires(), 1641034800);
    ASSERT_EQ(uut.authentication().feature_flags().is_vendor(), true);
    ASSERT_EQ(uut.authentication().feature_flags().is_white_labeled(), false);

    res = cpr::Delete(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}});

    ASSERT_EQ(res.status_code, cpr::status::HTTP_NO_CONTENT);

    ASSERT_EQ(uut.authentication().user().id(), 0);
    ASSERT_EQ(uut.authentication().user().user_email(), std::string{});
    ASSERT_EQ(uut.authentication().user().user_login(), std::string{});
    ASSERT_EQ(uut.authentication().user().display_name(), std::string{});
    ASSERT_EQ(uut.authentication().jwt().token(), std::string{});
    ASSERT_EQ(uut.authentication().jwt().token_expires(), 0);
    ASSERT_EQ(uut.authentication().feature_flags().is_vendor(), false);
    ASSERT_EQ(uut.authentication().feature_flags().is_white_labeled(), false);
}

TEST(console, deinit)
{
    uut.deinit();
    api.stop();
}
