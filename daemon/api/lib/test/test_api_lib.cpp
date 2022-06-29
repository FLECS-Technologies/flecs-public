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

#include <crow.h>

#include <filesystem>
#include <sstream>
#include <thread>

#include "gtest/gtest.h"
#include "libflecs.h"
#include "util/json/json.h"

#define TEST_PORT 18951

class echo_server_t
{
public:
    echo_server_t();

    auto stop() //
        -> void;

private:
    std::thread _server_thread;
};

static auto crow_app() //
    -> crow::SimpleApp&
{
    static auto app = crow::SimpleApp{};
    return app;
}

echo_server_t::echo_server_t()
    : _server_thread{}
{
    _server_thread = std::thread{[]() {
        CROW_CATCHALL_ROUTE(crow_app())
        ([](const crow::request& req) {
            auto response = crow::json::wvalue{};
            if ((req.method == crow::HTTPMethod::POST) || (req.method == crow::HTTPMethod::PUT))
            {
                response = crow::json::load(req.body);
            }
            response["endpoint"] = req.url;
            return crow::response(crow::status::OK, response.dump());
        });
        crow_app().port(TEST_PORT).loglevel(crow::LogLevel::Critical).run();
    }};
}

auto echo_server_t::stop() //
    -> void
{
    crow_app().stop();
    _server_thread.join();
}

static echo_server_t echo_server;
static const std::string app{"tech.flecs.test-app"};
static const std::string version{"1.2.3.4"};
static const std::string licenseKey{"valid-license-key"};
static const std::string instanceName{"some-instance-name"};
static const std::string instanceId{"56789abc"};

void stop_test_server()
{
    echo_server.stop();
}

TEST(api_lib, connect_tcp_success)
{
    auto lib = FLECS::libflecs_t{};
    const auto res = lib.connect("localhost", TEST_PORT);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/system/ping");
    ASSERT_EQ(lib.response_code(), crow::status::OK);
}

TEST(api_lib, connect_fail)
{
    testing::internal::CaptureStderr();

    auto lib = FLECS::libflecs_t{};
    const auto res = lib.connect("non-existent-host", TEST_PORT);
    const auto response = FLECS::parse_json(lib.json_response());

    testing::internal::GetCapturedStderr();

    ASSERT_EQ(res, -1);
}

TEST(api_lib, version)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.version();
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/system/version");
}

TEST(api_lib, app_install)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.install_app(app, version, licenseKey);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/install");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
    ASSERT_EQ(response["licenseKey"], licenseKey);
}

TEST(api_lib, app_uninstall)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.uninstall_app(app, version);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/uninstall");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, app_sideload_file_success)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto app_manifest = "app: tech.flecs.sideloaded-app\r\ntitle: Sideloaded FLECS app\r\n";
    const auto filename = "test-manifest.yml";

    auto manifest_file = fopen(filename, "w");
    fwrite(app_manifest, 1, strlen(app_manifest), manifest_file);
    fclose(manifest_file);
    const auto res = lib.sideload_app_from_file(filename);
    std::filesystem::remove(filename);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/sideload");
    ASSERT_EQ(response["appYaml"], app_manifest);
}

TEST(api_lib, app_sideload_string_success)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto app_manifest =
        "\"app\":\"ch.inasoft.sql4automation\",\"title\":\"SQL4AUTOMATION\",\"version\":\"v4.0.0.6\"";

    const auto res = lib.sideload_app_from_yaml(app_manifest);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/sideload");
    ASSERT_EQ(response["appYaml"], app_manifest);
}

TEST(api_lib, app_sideload_fail)
{
    testing::internal::CaptureStderr();

    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto filename = "non-existent-manifest.yml";

    const auto res = lib.sideload_app_from_file(filename);

    const auto response = FLECS::parse_json(lib.json_response());

    testing::internal::GetCapturedStderr();

    ASSERT_EQ(res, -1);
}

TEST(api_lib, app_list)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.list_apps();

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/list");
}

TEST(api_lib, instance_create)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.create_instance(app, version, instanceName);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/create");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
    ASSERT_EQ(response["instanceName"], instanceName);
}

TEST(api_lib, instance_start)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.start_instance(instanceId, app, version);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/start");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, instance_stop)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.stop_instance(instanceId, app, version);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/stop");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, delete_instance)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.delete_instance(instanceId, app, version);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/delete");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, ping)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto res = lib.ping();
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/system/ping");
}

TEST(api_lib, cmdline_version)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    auto argc = 2;
    char* argv[] = {(char*)"flecs", (char*)"version", nullptr};
    const auto res = lib.run_command(argc, argv);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/system/version");
}

TEST(api_lib, cmdline_app_install)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    auto argc = 6;
    char* argv[] = {
        (char*)"flecs",
        (char*)"app-manager",
        (char*)"install",
        (char*)app.c_str(),
        (char*)version.c_str(),
        (char*)licenseKey.c_str(),
        nullptr};

    const auto res = lib.run_command(argc, argv);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/install");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
    ASSERT_EQ(response["licenseKey"], licenseKey);
}

TEST(api_lib, cmdline_app_uninstall)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    auto args = std::vector<std::string>{"uninstall", app, version};
    const auto res = lib.run_command("app-manager", args);
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/uninstall");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, cmdline_app_sideload)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);
    const auto app_manifest = "app: tech.flecs.sideloaded-app\r\ntitle: Sideloaded FLECS app\r\n";
    const auto filename = "test-manifest.yml";

    auto manifest_file = fopen(filename, "w");
    fwrite(app_manifest, 1, strlen(app_manifest), manifest_file);
    fclose(manifest_file);
    const auto res = lib.run_command("app-manager", {"sideload", filename});
    std::filesystem::remove(filename);

    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/sideload");
    ASSERT_EQ(response["appYaml"], app_manifest);
}

TEST(api_lib, cmdline_app_list)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"list-apps"});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/app/list");
}

TEST(api_lib, cmdline_instance_create)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"create-instance", app, version, instanceName});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/create");
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
    ASSERT_EQ(response["instanceName"], instanceName);
}

TEST(api_lib, cmdline_instance_start)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"start-instance", instanceId, app, version});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/start");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, cmdline_instance_stop)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"stop-instance", instanceId, app, version});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/stop");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, cmdline_delete_instance)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"delete-instance", instanceId, app, version});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/instance/delete");
    ASSERT_EQ(response["instanceId"], instanceId);
    ASSERT_EQ(response["app"], app);
    ASSERT_EQ(response["version"], version);
}

TEST(api_lib, cmdline_ping)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("system", {"ping"});
    const auto response = FLECS::parse_json(lib.json_response());

    ASSERT_EQ(res, 0);
    ASSERT_EQ(response["endpoint"], "/system/ping");
}

TEST(api_lib, cmdline_unknown_1)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("unknown-command", {});

    ASSERT_EQ(res, -1);
}

TEST(api_lib, cmdline_unknown_2)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {"unknown-command"});

    ASSERT_EQ(res, -1);
}

TEST(api_lib, cmdline_incomplete)
{
    auto lib = FLECS::libflecs_t{};
    (void)lib.connect("localhost", TEST_PORT);

    const auto res = lib.run_command("app-manager", {});

    ASSERT_EQ(res, -1);
}

TEST(api_lib, shutdown)
{
    stop_test_server();
}
