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

                auto req_body = flecs::parse_json(req.body);
                // Activation via serial number
                if (req_body.contains("licenseKey")) {
                    auto license = req_body["licenseKey"].get<std::string>();
                    if (license == "UnknownLicense") {
                        const auto response = flecs::json_t({
                            {"statusCode", 403},
                            {"statusText", "Forbidden"},
                            {"reason", "License not found"},
                        });
                        return crow::response(403, response.dump());
                    } else if (license == "InvalidLicense") {
                        const auto response = flecs::json_t({
                            {"statusCode", 403},
                            {"statusText", "Forbidden"},
                            {"reason", "License found but invalid"},
                        });
                        return crow::response(403, response.dump());
                    } else if (license == "LicenseWithNoActivations") {
                        const auto response = flecs::json_t({
                            {"statusCode", 403},
                            {"statusText", "Forbidden"},
                            {"reason", "License found but maximum activations reached"},
                        });
                        return crow::response(403, response.dump());
                    } else if (license == "LicenseBreaksDatabase") {
                        const auto response = flecs::json_t({
                            {"statusCode", 500},
                            {"statusText", "Internal server error"},
                            {"reason", "Activation failed"},
                        });
                        return crow::response(500, response.dump());
                    } else if (license == "LicenseBreaksConsole1") {
                        const auto response = flecs::json_t({
                            {"some", 15},
                            {"random", "json"},
                            {"values", "response"},
                        });
                        return crow::response(500, response.dump());
                    } else if (license == "LicenseBreaksConsole2") {
                        const auto response = flecs::json_t({
                            {"invalid", true},
                            {"data", "for"},
                            {"code", 200},
                        });
                        return crow::response(200, response.dump());
                    }
                    const auto session_id = req.get_header_value("x-session-id");
                    if (session_id == "ValidSessionId") {
                        auto response = crow::response(204);
                        auto response_session_id = flecs::json_t({
                            {"id", session_id},
                            {"timestamp", 98465},
                        });
                        response.set_header("X-Session-id", response_session_id.dump());
                        return response;
                    }
                    const auto response = flecs::json_t({
                        {"data",
                            {
                                {"sessionId",
                                    {
                                        {"id", "NewValidSessionId"},
                                        {"timestamp", 729345},
                                    }
                                },
                                {"licenseKey", license},
                            }
                        },
                        {"statusCode", 200},
                        {"statusText", "Device successfully activated"},
                    });
                    return crow::response(200, response.dump());
                }

                // Activation via user credentials
                const auto auth = req.get_header_value("authorization").substr(7);
                if (auth.empty()) {
                    const auto response = flecs::json_t({
                        {"statusCode", 403},
                        {"statusText", "Forbidden"},
                        {"reason", "Invalid header: Authorization (expected Bearer)"},
                    });
                    return crow::response(403, response.dump());
                }
                auto test_data = auth;
                if (test_data == "200-valid") {
                    /* expected behavior for successful activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 200},
                        {"statusText", "OK"},
                        {"data",
                         {
                             {"sessionId", {{"id", test_data + "-session"}, {"timestamp", 0}}},
                             {"licenseKey", test_data + "-license"},
                         }},
                    });
                    return crow::response(200, response.dump());
                } else if (test_data == "200-invalid") {
                    /* unexpected response for successful activation */
                    return crow::response(200);
                } else if (test_data == "403") {
                    const auto response = flecs::json_t({
                        {"statusCode", 403},
                        {"statusText", "Forbidden"},
                        {"reason", "No remaining activations"},
                    });
                    return crow::response(403, response.dump());
                } else if (test_data == "500") {
                    /* expected behavior for errors during activation */
                    const auto response = flecs::json_t({
                        {"statusCode", 500},
                        {"statusText", "Internal Server Error"},
                        {"reason", "Could not retrieve device licenses"},
                    });
                    return crow::response(500, response.dump());
                } else if (test_data == "500-unhandled") {
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

auto put_auth_with_test_data(std::string test_data) //
    -> void
{
    auto auth = auth_response_json["data"];
    auth["jwt"]["token"] = test_data;
    cpr::Put(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}},
        cpr::Body{auth.dump()});
}

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
        cpr::Body{auth_response_json["data"].dump()});
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

TEST(console, activate_license_key)
{
    cpr::Delete(
        cpr::Url{"http://127.0.0.1:18951/v2/console/authentication"},
        cpr::Header{{{"Content-Type"}, {"application/json"}}});
    /** Valid sessionId, but user is not logged in */
    {
        const auto session_id = flecs::console::session_id_t{"200-valid", 0};
        const auto [error, result] = uut.activate_license_key();

        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Invalid header: Authorization (expected Bearer)");
    }

    /** Valid sessionId, and user is successfully logged in */
    {
        auto test_data =std::string("200-valid");
        put_auth_with_test_data(test_data);
        const auto [error, result] = uut.activate_license_key();
        ASSERT_FALSE(error.has_value());
        ASSERT_TRUE(result.has_value());
        ASSERT_EQ(result.value().session_id().id(), test_data + "-session");
        ASSERT_EQ(result.value().license_key(), test_data + "-license");
    }

    /** Valid sessionId, user is successfully logged in, but response is invalid */
    {
        put_auth_with_test_data("200-invalid");
        const auto [error, result] = uut.activate_license_key();

        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Invalid JSON response for status code 200");
    }

    /** No (unused) licenses available */
    {
        put_auth_with_test_data("403");
        const auto [error, result] = uut.activate_license_key();

        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "No remaining activations");
    }

    /** Server-side exception occurred during activation */
    {
        put_auth_with_test_data("500");
        const auto [error, result] = uut.activate_license_key();

        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Could not retrieve device licenses");
    }

    /** Unhandled server-side exception occurred during activation */
    {
        put_auth_with_test_data("500-unhandled");
        const auto [error, result] = uut.activate_license_key();

        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Activation failed with status code 500");
    }
}

TEST(console, activate_license)
{
    auto valid_session_id = flecs::console::session_id_t{"ValidSessionId", 34572};

    {
        auto [error, result] = uut.activate_license("UnknownLicense", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "License not found");
    }

    {
        auto [error, result] = uut.activate_license("InvalidLicense", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "License found but invalid");
    }

    {
        auto [error, result] = uut.activate_license("LicenseWithNoActivations", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "License found but maximum activations reached");
    }

    {
        auto [error, result] = uut.activate_license("LicenseBreaksDatabase", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Activation failed");
    }

    {
        auto [error, result] = uut.activate_license("LicenseBreaksConsole1", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Activation failed with status code 500");
    }

    {
        auto [error, result] = uut.activate_license("LicenseBreaksConsole2", {});
        ASSERT_TRUE(error.has_value());
        ASSERT_FALSE(result.has_value());
        ASSERT_EQ(error.value(), "Invalid JSON response for status code 200");
    }

    {
        auto [error, result] = uut.activate_license("AlreadyActiveLicense", {valid_session_id});
        if (error.has_value()) std::cerr << error.value() << "\n";
        ASSERT_FALSE(error.has_value());
        ASSERT_TRUE(result.has_value());
        ASSERT_EQ(result.value().session_id().id(), valid_session_id.id());
        ASSERT_EQ(result.value().session_id().timestamp(), 98465);
    }

    {
        auto [error, result] = uut.activate_license("ValidLicense", {});
        if (error.has_value()) std::cerr << error.value() << "\n";
        ASSERT_FALSE(error.has_value());
        ASSERT_TRUE(result.has_value());
        ASSERT_EQ(result.value().session_id().id(), "NewValidSessionId");
        ASSERT_EQ(result.value().session_id().timestamp(), 729345);
    }

    {
        auto arbitrary_session_id = flecs::console::session_id_t{"ArbitraryValidSessionId", 35078};
        auto [error, result] = uut.activate_license("ValidLicense", {arbitrary_session_id});
        if (error.has_value()) std::cerr << error.value() << "\n";
        ASSERT_FALSE(error.has_value());
        ASSERT_TRUE(result.has_value());
        ASSERT_EQ(result.value().session_id().id(), "NewValidSessionId");
        ASSERT_EQ(result.value().session_id().timestamp(), 729345);
    }
}

TEST(console, validate_license)
{
    /** SessionId is active */
    {
        const auto session_id = "200-active";
        const auto [res, message] = uut.validate_license(session_id);

        ASSERT_EQ(res, 1);
        ASSERT_EQ(message, "");
    }

    /** SessionId is inactive */
    {
        const auto session_id = "200-inactive";
        const auto [res, message] = uut.validate_license(session_id);

        ASSERT_EQ(res, 0);
        ASSERT_EQ(message, "");
    }

    /** SessionId is inactive. Invalid response from server */
    {
        const auto session_id = "200-invalid";
        const auto [res, message] = uut.validate_license(session_id);

        ASSERT_EQ(res, -1);
        ASSERT_EQ(message, "Invalid JSON response for status code 200");
    }

    /** Server-side exception occurred during validation */
    {
        const auto session_id = "500";
        const auto [res, message] = uut.validate_license(session_id);

        ASSERT_EQ(res, -1);
        ASSERT_EQ(message, "Could not retrieve device licenses");
    }

    /** Unhandled server-side exception occurred during activation */
    {
        const auto session_id = "500-unhandled";
        const auto [res, message] = uut.validate_license(session_id);

        ASSERT_EQ(res, -1);
        ASSERT_EQ(message, "Validation failed with status code 500");
    }
}

TEST(console, download_manifest)
{
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
