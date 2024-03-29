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

#include "flecs/modules/system/system.h"

class module_system_test_t : public flecs::module::system_t
{
public:
    module_system_test_t() = default;

    auto do_init() //
        -> void override
    {
        return flecs::module::system_t::do_init();
    }

    auto do_deinit() //
        -> void override
    {
        return flecs::module::system_t::do_deinit();
    }

    auto ping() const { return flecs::module::system_t::ping(); }
};

static auto uut = module_system_test_t{};

TEST(system, init)
{
    uut.do_init();

    flecs::flecs_api_t::instance().app().validate();
}

TEST(system, ping)
{
    using std::operator""s;

    auto req = crow::request{};
    auto res = crow::response{};

    const auto out_expected = R"({"additionalInfo":"OK"})"s;

    req.url = "/v2/system/ping";
    flecs::flecs_api_t::instance().app().handle(req, res);

    ASSERT_EQ(res.code, crow::status::OK);
    ASSERT_EQ(res.headers.find("Content-Type")->second, "application/json");
    ASSERT_EQ(res.body, out_expected);
}

TEST(system, info)
{
    auto req = crow::request{};
    auto res = crow::response{};

    req.url = "/v2/system/info";
    flecs::flecs_api_t::instance().app().handle(req, res);

    ASSERT_EQ(res.code, crow::status::OK);
    ASSERT_EQ(res.headers.find("Content-Type")->second, "application/json");
}

TEST(system, deinit)
{
    uut.do_deinit();
}
