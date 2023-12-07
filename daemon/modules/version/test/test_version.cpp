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

#include "gtest/gtest.h"
#include "util/json/json.h"
#include "version/version.h"

class test_module_version_t : public flecs::module::version_t
{
public:
    test_module_version_t() = default;

    auto do_init() //
        -> void override
    {
        return flecs::module::version_t::do_init();
    }

    auto do_deinit() //
        -> void override
    {
        return flecs::module::version_t::do_deinit();
    }
};

static auto uut = test_module_version_t{};

TEST(module_version, init)
{
    uut.do_init();

    flecs::flecs_api_t::instance().app().validate();
}

TEST(module_version, print_version)
{
    using std::operator""s;

    auto req = crow::request{};
    auto res = crow::response{};

    const auto json_expected = flecs::json_t({{"core", FLECS_VERSION + "-"s + FLECS_GIT_SHA}});

    req.url = "/v2/system/version";
    flecs::flecs_api_t::instance().app().handle(req, res);
    ASSERT_EQ(res.code, crow::status::OK);
    ASSERT_EQ(res.headers.find("Content-Type")->second, "application/json");
    ASSERT_EQ(res.body, json_expected.dump());
}

TEST(module_version, deinit)
{
    uut.do_deinit();
}
