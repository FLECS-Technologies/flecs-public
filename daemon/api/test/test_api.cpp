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

#include <thread>

#include "daemon/api/api.h"
#include "gtest/gtest.h"
#include "util/json/json.h"

class test_flecs_api_t
{
public:
    test_flecs_api_t()
    {
        FLECS_V2_ROUTE("/test/get").methods("GET"_method)([]() {
            auto response = flecs::json_t{};
            response["additionalInfo"] = "OK";
            return crow::response{crow::status::OK, response.dump()};
        });

        FLECS_V2_ROUTE("/test/post").methods("POST"_method)([](const crow::request& req) {
            auto response = flecs::json_t{};
            const auto args = flecs::parse_json(req.body);
            if (!args.contains("arg")) {
                return crow::response(crow::status::BAD_REQUEST);
            }
            response["additionalInfo"] = "OK";

            return crow::response(crow::status::OK, response.dump());
        });

        auto& api = flecs::flecs_api_t::instance();
        auto& app = api.app().port(8951);
        app.loglevel(crow::LogLevel::CRITICAL);
        _api_thread = std::thread{&crow::SimpleApp::run, &app};
    }

    ~test_flecs_api_t()
    {
        flecs::flecs_api_t::instance().app().stop();
        _api_thread.join();
    }

private:
    std::thread _api_thread;
};

static auto test_api = test_flecs_api_t{};

TEST(api, tcp_socket_not_found)
{
    const auto res = cpr::Get(cpr::Url{"http://localhost:8951/v2/test/invalid_endpoint"});

    ASSERT_EQ(res.status_code, crow::status::NOT_FOUND);
}

TEST(api, endpoint_get)
{
    const auto res = cpr::Get(cpr::Url{"http://localhost:8951/v2/test/get"});

    ASSERT_EQ(res.status_code, crow::status::OK);
    ASSERT_EQ(res.text, R"({"additionalInfo":"OK"})");
}

TEST(api, endpoint_post)
{
    auto json = flecs::json_t{};
    json["arg"] = "value";

    const auto res = cpr::Post(
        cpr::Url{"http://localhost:8951/v2/test/post"},
        cpr::Body{json.dump()},
        cpr::Header{{"Content-Type", "application/json"}});

    ASSERT_EQ(res.status_code, crow::status::OK);
    ASSERT_EQ(res.text, R"({"additionalInfo":"OK"})");
}

TEST(api, bad_request)
{
    const auto res = cpr::Post(
        cpr::Url{"http://localhost:8951/v2/test/post"},
        cpr::Body{"Not a JSON body"},
        cpr::Header{{"Content-Type", "application/json"}});

    ASSERT_EQ(res.status_code, crow::status::BAD_REQUEST);
}

TEST(api, not_allowed)
{
    const auto res = cpr::Patch(cpr::Url{"http://localhost:8951/v2/test/get"});

    ASSERT_EQ(res.status_code, crow::status::METHOD_NOT_ALLOWED);
}
