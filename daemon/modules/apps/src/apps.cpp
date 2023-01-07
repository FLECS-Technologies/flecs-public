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

#include "apps.h"

#include <archive.h>
#include <archive_entry.h>

#include "factory/factory.h"
#include "impl/apps_impl.h"
#include "util/datetime/datetime.h"

namespace FLECS {

namespace {
register_module_t<module_apps_t> _reg("apps");
}

module_apps_t::module_apps_t()
    : _impl{new impl::module_apps_impl_t{this}}
{}

module_apps_t::~module_apps_t()
{
    save();
}

auto module_apps_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/apps").methods("GET"_method)([this]() { return list(); });

    FLECS_V2_ROUTE("/apps/<string>").methods("GET"_method)([this](const crow::request& req, const std::string& app) {
        const auto version = req.url_params.get("version");
        if (version) {
            return list(app, version);
        } else {
            return list(app, {});
        }
    });

    FLECS_V2_ROUTE("/apps/<string>").methods("DELETE"_method)([this](const crow::request& req, const std::string& app) {
        const auto version = req.url_params.get("version");
        if (version) {
            return uninstall(app, version, false);
        } else {
            return uninstall(app, {}, false);
        }
    });

    FLECS_V2_ROUTE("/apps/install").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, app);
        REQUIRED_JSON_VALUE(args, version);
        OPTIONAL_JSON_VALUE(args, licenseKey);
        return install_from_marketplace(app, version, licenseKey);
    });

    FLECS_V2_ROUTE("/apps/sideload").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, appYaml);
        OPTIONAL_JSON_VALUE(args, licenseKey);
        return sideload(appYaml, licenseKey);
    });

#if 0
    FLECS_V2_ROUTE("/apps/export").methods("POST"_method)([this](const crow::request& req) {
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

    response["additionalInfo"] = "OK";
    return crow::response{crow::status::OK, response.dump()};
});
#endif // 0

    load();

    return _impl->do_init();
}

auto module_apps_t::load(fs::path base_path) //
    -> crow::response
{
    return _impl->do_load(std::move(base_path));
}

auto module_apps_t::save(fs::path base_path) const //
    -> crow::response
{
    return _impl->do_save(std::move(base_path));
}

auto module_apps_t::list(std::string_view app_name, std::string_view version) const //
    -> crow::response
{
    return _impl->do_list(std::move(app_name), std::move(version));
}
auto module_apps_t::list(std::string_view app_name) const //
    -> crow::response
{
    return list(std::move(app_name), {});
}
auto module_apps_t::list() const //
    -> crow::response
{
    return list({}, {});
}

auto module_apps_t::install_from_marketplace(std::string app_name, std::string version, std::string license_key) //
    -> crow::response
{
    return _impl->queue_install_from_marketplace(std::move(app_name), std::move(version), std::move(license_key));
}
auto module_apps_t::install_from_marketplace(std::string app_name, std::string version) //
    -> crow::response
{
    return install_from_marketplace(std::move(app_name), std::move(version), {});
}

auto module_apps_t::sideload(std::string manifest_string, std::string license_key) //
    -> crow::response
{
    return _impl->queue_sideload(std::move(manifest_string), std::move(license_key));
}
auto module_apps_t::sideload(std::string manifest_string) //
    -> crow::response
{
    return sideload(std::move(manifest_string), {});
}

auto module_apps_t::uninstall(std::string app_name, std::string version, bool force) //
    -> crow::response
{
    return _impl->queue_uninstall(std::move(app_name), std::move(version), std::move(force));
}

auto module_apps_t::export_app(std::string app_name, std::string version) const //
    -> crow::response
{
    return _impl->queue_export_app(std::move(app_name), std::move(version));
}
auto module_apps_t::export_app(std::string app_name) const //
    -> crow::response
{
    return export_app(std::move(app_name), {});
}

auto module_apps_t::has_app(std::string_view app_name, std::string_view version) const noexcept //
    -> bool
{
    return _impl->has_app(std::move(app_name), std::move(version));
}

auto module_apps_t::is_app_installed(std::string_view app_name, std::string_view version) const noexcept //
    -> bool
{
    return _impl->is_app_installed(std::move(app_name), std::move(version));
}

} // namespace FLECS
