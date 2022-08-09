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

#include "app_manager.h"

#include "deployment/deployment_docker.h"
#include "factory/factory.h"
#include "instance/instance_config.h"
#include "private/app_manager_private.h"
#include "util/string/comparator.h"

namespace FLECS {

namespace {
register_module_t<module_app_manager_t> _reg("app-manager");
}

module_app_manager_t::module_app_manager_t()
    : _impl{new Private::module_app_manager_private_t}
{}

module_app_manager_t::~module_app_manager_t()
{}

auto module_app_manager_t::do_init() //
    -> void
{
    FLECS_ROUTE("/app/install").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app);
        REQUIRED_JSON_VALUE(args, version);
        OPTIONAL_JSON_VALUE(args, licenseKey);
        const auto status = _impl->do_install(app, version, licenseKey, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/app/instances").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app_name);
        OPTIONAL_JSON_VALUE(args, version);
        const auto status = _impl->do_list_instances(app_name, version, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/app/list").methods("GET"_method)([=]() {
        auto response = json_t{};
        const auto status = _impl->do_list_apps(response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/app/sideload").methods("PUT"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, appYaml);
        OPTIONAL_JSON_VALUE(args, licenseKey);
        const auto status = _impl->do_sideload(appYaml, licenseKey, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/app/uninstall").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app);
        REQUIRED_JSON_VALUE(args, version);
        const auto status = _impl->do_uninstall(app, version, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/app/versions").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app_name);
        const auto status = _impl->do_list_versions(app_name, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/config").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        const auto status = _impl->do_get_config_instance(instanceId, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/<string>/instance/<string>/config")
        .methods("GET"_method)([=](const std::string& /*api_version*/, const std::string& instance_id) {
            auto response = json_t{};
            const auto status = _impl->do_get_config_instance(instance_id, response);
            return crow::response{status, response.dump()};
        });

    FLECS_ROUTE("/instance/config").methods("PUT"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        auto config = instance_config_t{};
        if (args.contains("networkAdapters"))
        {
            args["networkAdapters"].get_to(config.networkAdapters);
        }
        if (args.contains("devices") && args["devices"].contains("usb"))
        {
            args["devices"]["usb"].get_to(config.usb_devices);
        }

        const auto status = _impl->do_put_config_instance(instanceId, config, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/<path>/instance/<path>/config")
        .methods("PUT"_method)(
            [=](const crow::request& req, const std::string& /*api_version*/, const std::string& instance_id) {
                auto response = json_t{};
                const auto args = parse_json(req.body);
                auto config = instance_config_t{};
                if (args.contains("networkAdapters"))
                {
                    args["networkAdapters"].get_to(config.networkAdapters);
                }
                if (args.contains("devices") && args["devices"].contains("usb"))
                {
                    args["devices"]["usb"].get_to(config.usb_devices);
                }

                const auto status = _impl->do_put_config_instance(instance_id, config, response);
                return crow::response{status, response.dump()};
            });

    FLECS_ROUTE("/instance/create").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app);
        REQUIRED_JSON_VALUE(args, version);
        OPTIONAL_JSON_VALUE(args, instanceName);
        const auto status = _impl->do_create_instance(app, version, instanceName, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/delete").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        OPTIONAL_JSON_VALUE(args, app);
        OPTIONAL_JSON_VALUE(args, version);
        const auto status = _impl->do_delete_instance(instanceId, app, version, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/details").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        const auto status = _impl->do_instance_details(instanceId, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/log").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        const auto status = _impl->do_instance_log(instanceId, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/start").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        OPTIONAL_JSON_VALUE(args, app);
        OPTIONAL_JSON_VALUE(args, version);
        const auto status = _impl->do_start_instance(instanceId, app, version, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/instance/stop").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, instanceId);
        OPTIONAL_JSON_VALUE(args, app);
        OPTIONAL_JSON_VALUE(args, version);
        const auto status = _impl->do_stop_instance(instanceId, app, version, response);
        return crow::response{status, response.dump()};
    });

    return _impl->do_init();
}

} // namespace FLECS
