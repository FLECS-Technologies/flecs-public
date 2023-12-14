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

#include "daemon/modules/apps/apps.h"

#include "daemon/common/app/manifest/manifest.h"
#include "daemon/modules/apps/impl/apps_impl.h"
#include "daemon/modules/factory/factory.h"
#include "util/datetime/datetime.h"

namespace flecs {
namespace module {

namespace {
register_module_t<apps_t> _reg("apps");
}

apps_t::apps_t()
    : _impl{new impl::apps_t{this}}
{}

apps_t::~apps_t()
{}

auto apps_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/apps").methods("GET"_method)([this]() { return http_list({}); });

    FLECS_V2_ROUTE("/apps/<string>").methods("GET"_method)([this](const crow::request& req, std::string app) {
        const auto version = req.url_params.get("version");
        if (version) {
            return http_list(apps::key_t{std::move(app), version});
        } else {
            return http_list(apps::key_t{std::move(app), {}});
        }
    });

    FLECS_V2_ROUTE("/apps/<string>")
        .methods("DELETE"_method)([this](const crow::request& req, std::string app) {
            const auto version = req.url_params.get("version");
            return http_uninstall(apps::key_t{std::move(app), version ? version : ""});
        });

    FLECS_V2_ROUTE("/apps/install").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON_VALUE(args, appKey, apps::key_t);
        return http_install(std::move(appKey));
    });

    FLECS_V2_ROUTE("/apps/sideload").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, manifest);
        return http_sideload(std::move(manifest));
    });

    _impl->do_module_init();
}

auto apps_t::do_load(const fs::path& base_path) //
    -> result_t
{
    return _impl->do_load(base_path / "apps");
}

auto apps_t::do_start() //
    -> void
{
    return _impl->do_module_start();
}

auto apps_t::do_save(const fs::path& base_path) const //
    -> result_t
{
    return _impl->do_save(base_path / "apps");
}

auto apps_t::http_list(const apps::key_t& app_key) const //
    -> crow::response
{
    auto response = json_t::array();

    const auto keys = app_keys(app_key);
    for (const auto& key : keys) {
        auto app = query(key);
        if (app) {
            response.push_back(*app);
            /** @todo this should be done some other way */
            auto& val = *response.rbegin();
            if (auto manifest = app->manifest()) {
                val["multiInstance"] = manifest->multi_instance();
                val["editor"] = manifest->editor();
            } else {
                val["multiInstance"] = false;
                val["editor"] = std::string{};
            }
        }
    }

    return {crow::status::OK, "json", response.dump()};
}

auto apps_t::http_install(apps::key_t app_key) //
    -> crow::response
{
    auto job_id = _impl->queue_install_from_marketplace(std::move(app_key));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto apps_t::http_sideload(std::string manifest_string) //
    -> crow::response
{
    auto job_id = _impl->queue_sideload(std::move(manifest_string));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto apps_t::http_uninstall(apps::key_t app_key) //
    -> crow::response
{
    if (!is_installed(app_key)) {
        auto response = json_t{};
        response["additionalInfo"] = "Cannot uninstall " + to_string(app_key) + ", which is not installed";
        return {crow::status::BAD_REQUEST, "json", response.dump()};
    }

    auto job_id = _impl->queue_uninstall(std::move(app_key));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto apps_t::http_export_to(apps::key_t app_key) //
    -> crow::response
{
    if (!is_installed(app_key)) {
        auto response = json_t{};
        response["additionalInfo"] = "Cannot export " + to_string(app_key) + ", which is not installed";
        return {crow::status::BAD_REQUEST, "json", response.dump()};
    }

    auto job_id = _impl->queue_export_to(std::move(app_key), "/var/lib/flecs/exports/");
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto apps_t::app_keys(const apps::key_t& app_key) const //
    -> std::vector<apps::key_t>
{
    return _impl->do_app_keys(app_key);
}

auto apps_t::app_keys(std::string app_name, std::string version) const //
    -> std::vector<apps::key_t>
{
    return app_keys(apps::key_t{std::move(app_name), std::move(version)});
}
auto apps_t::app_keys(std::string app_name) const //
    -> std::vector<apps::key_t>
{
    return app_keys(apps::key_t{std::move(app_name), {}});
}
auto apps_t::app_keys() const //
    -> std::vector<apps::key_t>
{
    return app_keys(apps::key_t{});
}

auto apps_t::install_from_marketplace(apps::key_t app_key) //
    -> result_t
{
    return _impl->do_install_from_marketplace_sync(std::move(app_key));
}

auto apps_t::sideload(std::string manifest_string) //
    -> result_t
{
    return _impl->do_sideload_sync(std::move(manifest_string));
}

auto apps_t::uninstall(apps::key_t app_key) //
    -> result_t
{
    return _impl->do_uninstall_sync(std::move(app_key));
}

auto apps_t::export_to(apps::key_t app_key, fs::path dest_dir) const //
    -> result_t
{
    return _impl->do_export_to_sync(std::move(app_key), std::move(dest_dir));
}

auto apps_t::import_from(apps::key_t app_key, fs::path src_dir) //
    -> result_t
{
    return _impl->do_import_from_sync(std::move(app_key), std::move(src_dir));
}

auto apps_t::query(const apps::key_t& app_key) const noexcept //
    -> std::shared_ptr<apps::app_t>
{
    return _impl->do_query(app_key);
}

auto apps_t::is_installed(const apps::key_t& app_key) const noexcept //
    -> bool
{
    return _impl->do_is_installed(app_key);
}

} // namespace module
} // namespace flecs
