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

#include <archive.h>
#include <archive_entry.h>

#include <fstream>

#include "deployment/deployment_docker.h"
#include "factory/factory.h"
#include "instance/instance_config.h"
#include "private/app_manager_private.h"
#include "util/datetime/datetime.h"
#include "util/string/comparator.h"
#include "util/string/literals.h"

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

    FLECS_ROUTE("/<string>/app/exports")
        .methods("POST"_method)([=](const crow::request& req, const std::string& /* api_version */) {
            auto response = json_t{};
            const auto args = parse_json(req.body);

            if (!args.contains("apps") && !args.contains("instances")) {
                return crow::response{crow::status::OK, response.dump()};
            }

            auto tmpdir = tmpdir_t{"/var/lib/flecs/export-tmp/"};
            if (!tmpdir.created()) {
                response["additionalInfo"] = "Could not create export-tmp directory";
                return crow::response{crow::status::INTERNAL_SERVER_ERROR, response.dump()};
            }

            /* export apps */
            if (args.contains("apps")) {
                for (const auto& j : args["apps"]) {
                    REQUIRED_JSON_VALUE(j, app);
                    REQUIRED_JSON_VALUE(j, version);
                    auto res = _impl->do_export_app(app, version);
                    if (res.code != crow::status::OK) {
                        return res;
                    }
                }
            }
            /* export instances */
            if (args.contains("instances")) {
                for (const auto& j : args["instances"]) {
                    REQUIRED_JSON_VALUE(j, instanceId);
                    auto res = _impl->do_export_instance(instanceId);
                    if (res.code != crow::status::OK) {
                        return res;
                    }
                }
            }
            /* export deployment */
            _impl->do_save("/var/lib/flecs/export-tmp/");

            /* create export package */
            auto ec = std::error_code{};
            fs::create_directories("/var/lib/flecs/exports", ec);
            if (ec) {
                response["additionalInfo"] = "Could not create exports directory";
                return crow::response{crow::status::INTERNAL_SERVER_ERROR, response.dump()};
            }

            using std::operator""s;
            auto outname = "/var/lib/flecs/exports/"s.append(unix_time(precision_e::seconds)).append(".tar.gz");
            auto archive = archive_write_new();
            archive_write_add_filter_gzip(archive);
            archive_write_set_format_pax_restricted(archive);
            archive_write_open_filename(archive, outname.c_str());

            auto entry = archive_entry_new();
            auto buf = std::unique_ptr<char[]>{new char[1_MiB]};
            for (const auto& file : fs::recursive_directory_iterator("/var/lib/flecs/export-tmp", ec)) {
                if (ec) {
                    break;
                }
                if (file.status().type() != fs::file_type::regular) {
                    continue;
                }
                const auto relpath = fs::relative(file.path(), "/var/lib/flecs/export-tmp/", ec);
                if (ec) {
                    break;
                }
                archive_entry_set_pathname(entry, relpath.c_str());
                archive_entry_set_size(entry, file.file_size());
                archive_entry_set_filetype(entry, AE_IFREG);
                archive_entry_set_perm(entry, 0644);
                archive_write_header(archive, entry);
                auto f = std::ifstream{file.path().c_str(), std::ios_base::in | std::ios_base::binary};
                f.read(buf.get(), 1_MiB);
                while (f.gcount()) {
                    archive_write_data(archive, buf.get(), f.gcount());
                    f.read(buf.get(), 1_MiB);
                }
                archive_entry_clear(entry);
            }
            archive_entry_free(entry);
            archive_write_close(archive);
            archive_write_free(archive);

            response["additionalInfo"] = "OK";
            return crow::response{crow::status::OK, response.dump()};
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
        if (args.contains("networkAdapters")) {
            args["networkAdapters"].get_to(config.networkAdapters);
        }
        if (args.contains("devices") && args["devices"].contains("usb")) {
            args["devices"]["usb"].get_to(config.usb_devices);
        }

        const auto status = _impl->do_put_config_instance(instanceId, config, response);
        return crow::response{status, response.dump()};
    });

    FLECS_ROUTE("/<string>/instance/<string>/config")
        .methods("PUT"_method)(
            [=](const crow::request& req, const std::string& /*api_version*/, const std::string& instance_id) {
                auto response = json_t{};
                const auto args = parse_json(req.body);
                auto config = instance_config_t{};
                if (args.contains("networkAdapters")) {
                    args["networkAdapters"].get_to(config.networkAdapters);
                }
                if (args.contains("devices") && args["devices"].contains("usb")) {
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

    FLECS_ROUTE("/<string>/instance/<string>/update")
        .methods("PUT"_method)(
            [=](const crow::request& req, const std::string& /*api_version*/, const std::string& instance_id) {
                auto response = json_t{};
                const auto args = parse_json(req.body);
                OPTIONAL_JSON_VALUE(args, app);
                OPTIONAL_JSON_VALUE(args, from);
                REQUIRED_JSON_VALUE(args, to);
                const auto status = _impl->do_update_instance(instance_id, app, from, to, response);
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
