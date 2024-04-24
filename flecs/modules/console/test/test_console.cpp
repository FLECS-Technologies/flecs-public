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
#include <gtest/gtest.h>

#include "flecs/modules/console/console.h"
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

class mock_console_t
{
public:
    static const char* manifest;

    mock_console_t()
        : _app{}
        , _{}
    {}

    auto init() //
        -> void
    {
        CROW_ROUTE(_app, "/api/v2/device/license/activate")
            .methods("POST"_method)([](const crow::request& req) {
                auto response = flecs::json_t{};
                const auto auth = req.get_header_value("authorization").substr(7);
                if (auth.empty()) {
                    const auto response = flecs::json_t({
                        {"statusCode", 403},
                        {"statusText", "Forbidden"},
                        {"reason", "Invalid header: Authorization (expected Bearer)"},
                    });
                    return crow::response(403, response.dump());
                }

                const auto session_id = req.get_header_value("x-session-id");
                if (session_id == "200-valid") {
                    /* expected behavior for successful activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 200},
                        {"statusText", "OK"},
                        {"data",
                         {
                             {"sessionId", session_id},
                         }},
                    });
                    return crow::response(200, response.dump());
                } else if (session_id == "200-invalid") {
                    /* unexpected response for successful activation */
                    return crow::response(200);
                } else if (session_id == "204") {
                    /* expected behavior for already active device */
                    return crow::response(204);
                } else if (session_id == "403") {
                    const auto response = flecs::json_t({
                        {"statusCode", 403},
                        {"statusText", "Forbidden"},
                        {"reason", "No remaining activations"},
                    });
                    return crow::response(500, response.dump());
                } else if (session_id == "500") {
                    /* expected behavior for errors during activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 500},
                        {"statusText", "Internal Server Error"},
                        {"reason", "Could not retrieve device licenses"},
                    });
                    return crow::response(500, response.dump());
                } else if (session_id == "500-unhandled") {
                    /* unexpected behavior, unhandled error during activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 500},
                        {"statusText", "Internal Server Error"},
                    });
                    return crow::response(500, response.dump());
                }

                return crow::response{500, std::string{}};
            });

        CROW_ROUTE(_app, "/api/v2/device/license/validate")
            .methods("POST"_method)([](const crow::request& req) {
                auto response = flecs::json_t{};
                const auto auth = req.get_header_value("authorization").substr(7);
                if (auth.empty()) {
                    const auto response = flecs::json_t({
                        {"statusCode", 403},
                        {"statusText", "Forbidden"},
                        {"reason", "Invalid header: Authorization (expected Bearer)"},
                    });
                    return crow::response(403, response.dump());
                }

                const auto session_id = req.get_header_value("x-session-id");
                if (session_id == "200-active") {
                    /* expected behavior for successful validation of active device */
                    const auto response = flecs::json_t({
                        {"statusCode", 200},
                        {"statusText", "OK"},
                        {"data",
                         {
                             {"isValid", true},
                         }},
                    });
                    return crow::response(200, response.dump());
                } else if (session_id == "200-inactive") {
                    /* expected behavior for successful validation of inactive device */
                    const auto response = flecs::json_t({
                        {"statusCode", 200},
                        {"statusText", "OK"},
                        {"data",
                         {
                             {"isValid", false},
                         }},
                    });
                    return crow::response(200, response.dump());
                } else if (session_id == "200-invalid") {
                    /* unexpected response for successful validation */
                    return crow::response(200);
                } else if (session_id == "500") {
                    /* expected behavior for errors during activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 500},
                        {"statusText", "Internal Server Error"},
                        {"reason", "Could not retrieve device licenses"},
                    });
                    return crow::response(500, response.dump());
                } else if (session_id == "500-unhandled") {
                    /* unexpected behavior, unhandled error during activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 500},
                        {"statusText", "Internal Server Error"},
                    });
                    return crow::response(500, response.dump());
                }

                return crow::response{500, std::string{}};
            });

        CROW_ROUTE(_app, "/api/v2/manifests/<string>/<string>")
            .methods("GET"_method)(
                [](const crow::request& req, const std::string& app, const std::string& version) {
                    const auto auth = req.get_header_value("authorization").substr(7);
                    if (auth.empty()) {
                        const auto response = flecs::json_t({
                            {"statusCode", 403},
                            {"statusText", "Forbidden"},
                            {"reason", "Invalid header: Authorization (expected Bearer)"},
                        });
                        return crow::response(403, response.dump());
                    }

                    if (app != "app" || version != "version") {
                        throw;
                    }
                    const auto session_id = req.get_header_value("x-session-id");
                    if (session_id == "200-valid") {
                        const auto json_manifest = flecs::parse_json(manifest);
                        const auto response = flecs::json_t({
                            {"statusCode", 200},
                            {"statusText", "OK"},
                            {"data", json_manifest},
                        });
                        return crow::response(200, response.dump());
                    }

                    if (session_id == "404-notfound") {
                        const auto response = flecs::json_t({
                            {"statusCode", 404},
                            {"statusText", "Not Found"},
                        });
                        return crow::response(404, response.dump());
                    }

                    if (session_id == "500") {
                        const auto response = flecs::json_t({
                            {"statusCode", 500},
                            {"statusText", "Internal Server Error"},
                            {"reason", "Could not retrieve App Manifest"},
                        });
                        return crow::response(500, response.dump());
                    }

                    if (session_id == "500-unhandled") {
                        /* unexpected behavior, unhandled error during activation */
                        const auto response = flecs::json_t({
                            {"statusCode", 500},
                            {"statusText", "Internal Server Error"},
                        });
                        return crow::response(500, response.dump());
                    }

                    return crow::response{500, std::string{}};
                });
    }

    auto start() //
        -> void
    {
        _ = _app.loglevel(crow::LogLevel::Critical).bindaddr("127.0.0.1").port(18952).run_async();
        _app.wait_for_server_start();
    }

    auto stop() //
        -> void
    {
        _app.stop();
        _.wait();
    }

private:
    crow::SimpleApp _app;
    std::future<void> _;
};

const char* mock_console_t::manifest = R"-(
    {
        "app": "tech.flecs.test-app-1",
        "version": "1.2.3.4-f1",
        "image": "flecs/tech.flecs.test-app-1"
    }
)-";

static constexpr auto user = "testuser";
static constexpr auto token = "abcdef-1234-5678-XYZ";

static auto api = test_api_t{};
static auto console = mock_console_t{};
static auto uut = module_console_test_t{};

TEST(console, init)
{
    console.init();
    uut.init();
    console.start();
    api.start();
}

TEST(console, base_url)
{
    const auto url = uut.base_url();

    ASSERT_EQ(url, "http://127.0.0.1:18952");
}

TEST(console, download_manifest)
{
    cpr::Delete(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}});
    /** Valid App, but user is not logged in */
    {
        const auto session_id = "200-valid";
        const auto expected = std::string{};
        const auto actual = uut.download_manifest("app", "version", session_id);

        ASSERT_EQ(actual, expected);
    }

    cpr::Put(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}},
        cpr::Body{auth_response_json["data"].dump()});
    /** User logged in, sessionId is active */
    {
        const auto session_id = "200-valid";
        const auto expected = std::string{mock_console_t::manifest};
        const auto actual = uut.download_manifest("app", "version", session_id);

        ASSERT_EQ(flecs::parse_json(actual), flecs::parse_json(actual));
    }
}

TEST(console, deinit)
{
    api.stop();
    console.stop();
    uut.deinit();
}
