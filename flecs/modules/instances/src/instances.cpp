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

#include "flecs/modules/instances/instances.h"

#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/instances/impl/instances_impl.h"
#include "flecs/modules/instances/types/instance.h"

namespace flecs {
namespace module {

namespace {
register_module_t<instances_t> _reg("instances");
}

instances_t::instances_t()
    : _impl{new impl::instances_t{this}}
{}

instances_t::~instances_t()
{}

auto instances_t::do_load(const fs::path& base_path) //
    -> result_t
{
    return _impl->do_load(base_path);
}

auto instances_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/instances").methods("GET"_method)([this](const crow::request& req) {
        auto app = req.url_params.get("app");
        auto version = req.url_params.get("version");
        return http_list(apps::key_t{app ? app : "", version ? version : ""});
    });

    FLECS_V2_ROUTE("/instances/<string>").methods("GET"_method)([this](const std::string& instance_id) {
        return http_details(instances::id_t{instance_id});
    });

    FLECS_V2_ROUTE("/instances/create").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON_VALUE(args, appKey, apps::key_t);
        OPTIONAL_JSON_VALUE(args, instanceName);
        return http_create(std::move(appKey), std::move(instanceName), false);
    });

    FLECS_V2_ROUTE("/instances/<string>")
        .methods("PATCH"_method)([this](const crow::request& req, const std::string& instance_id) {
            auto response = json_t{};
            const auto args = parse_json(req.body);
            REQUIRED_JSON_VALUE(args, to);
            return http_update(instances::id_t{instance_id}, std::move(to));
        });

    FLECS_V2_ROUTE("/instances/<string>").methods("DELETE"_method)([this](const std::string& instance_id) {
        return http_remove(instances::id_t{instance_id});
    });

    FLECS_V2_ROUTE("/instances/<string>/start")
        .methods("POST"_method)(
            [this](const std::string& instance_id) { return http_start(instances::id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/<string>/stop").methods("POST"_method)([this](const std::string& instance_id) {
        return http_stop(instances::id_t{instance_id});
    });

    FLECS_V2_ROUTE("/instances/<string>/config")
        .methods("GET"_method)(
            [this](const std::string& instance_id) { return http_get_config(instances::id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/<string>/config")
        .methods("POST"_method)([this](const crow::request& req, const std::string& instance_id) {
            auto response = json_t{};
            const auto args = parse_json(req.body);
            auto config = instances::config_t{};
            if (args.contains("networkAdapters")) {
                args["networkAdapters"].get_to(config.networkAdapters);
            }
            if (args.contains("devices") && args["devices"].contains("usb")) {
                args["devices"]["usb"].get_to(config.usb_devices);
            }

            return http_post_config(instances::id_t{instance_id}, config);
        });

    FLECS_V2_ROUTE("/instances/<string>/logs").methods("GET"_method)([this](const std::string& instance_id) {
        return http_logs(instances::id_t{instance_id});
    });

    return _impl->do_module_init();
}

auto instances_t::do_start() //
    -> void
{
    return _impl->do_module_start();
}

auto instances_t::do_stop() //
    -> void
{
    return _impl->do_module_stop();
}

auto instances_t::http_list(const apps::key_t& app_key) const //
    -> crow::response
{
    auto response = json_t::array();

    for (const auto& instance_id : instance_ids(app_key)) {
        auto json = json_t::object();

        auto instance = query(instance_id);
        json["instanceId"] = instance->id().hex();
        json["instanceName"] = instance->instance_name();
        json["appKey"] = apps::key_t{instance->app_name().data(), instance->app_version().data()};
        auto app = instance->app();
        if (!app || app->status() == apps::status_e::Orphaned) {
            json["status"] = to_string(instances::status_e::Orphaned);
        } else {
            if (instance->status() == instances::status_e::Created) {
                json["status"] = to_string(
                    is_running(instance) ? instances::status_e::Running : instances::status_e::Stopped);
            } else {
                json["status"] = to_string(instance->status());
            }
        }
        json["desired"] = to_string(instance->desired());
        response.push_back(std::move(json));
    }

    return {crow::status::OK, "json", response.dump()};
}

auto instances_t::http_details(instances::id_t instance_id) const //
    -> crow::response
{
    return _impl->do_details(std::move(instance_id));
}

auto instances_t::http_create(apps::key_t app_key, std::string instance_name, bool running) //
    -> crow::response
{
    auto job_id = _impl->queue_create(std::move(app_key), std::move(instance_name), running);
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::http_start(instances::id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_start(std::move(instance_id), false);
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::http_stop(instances::id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_stop(std::move(instance_id), false);
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::http_remove(instances::id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_remove(std::move(instance_id));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::http_get_config(instances::id_t instance_id) const //
    -> crow::response
{
    return _impl->do_get_config(std::move(instance_id));
}

auto instances_t::http_post_config(instances::id_t instance_id, const instances::config_t& config) //
    -> crow::response
{
    return _impl->do_post_config(std::move(instance_id), config);
}

auto instances_t::http_logs(instances::id_t instance_id) const //
    -> crow::response
{
    return _impl->do_logs(std::move(instance_id));
}

auto instances_t::http_update(instances::id_t instance_id, std::string to) //
    -> crow::response
{
    auto job_id = _impl->queue_update(std::move(instance_id), std::move(to));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::http_export_to(instances::id_t instance_id, fs::path dest_dir) const //
    -> crow::response
{
    auto job_id = _impl->queue_export_to(std::move(instance_id), std::move(dest_dir));
    return crow::response{crow::status::ACCEPTED, "json", "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto instances_t::instance_ids(const apps::key_t& app_key) const //
    -> std::vector<instances::id_t>
{
    return _impl->do_instance_ids(app_key);
}
auto instances_t::instance_ids(std::string app_name, std::string version) const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{std::move(app_name), std::move(version)});
}
auto instances_t::instance_ids(std::string app_name) const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{std::move(app_name), {}});
}
auto instances_t::instance_ids() const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{});
}

auto instances_t::query(instances::id_t instance_id) const //
    -> std::shared_ptr<instances::instance_t>
{
    return _impl->do_query(std::move(instance_id));
}

auto instances_t::is_running(std::shared_ptr<instances::instance_t> instance) const //
    -> bool
{
    return _impl->do_is_running(std::move(instance));
}

auto instances_t::create(apps::key_t app_key, std::string instance_name, bool running) //
    -> result_t
{
    return _impl->do_create_sync(std::move(app_key), std::move(instance_name), running);
}
auto instances_t::create(apps::key_t app_key) //
    -> result_t
{
    return create(std::move(app_key), {}, false);
}
auto instances_t::create(std::string app_name, std::string version, std::string instance_name) //
    -> result_t
{
    return create(apps::key_t{std::move(app_name), std::move(version)}, std::move(instance_name), false);
}
auto instances_t::create(std::string app_name, std::string version) //
    -> result_t
{
    return create(apps::key_t{std::move(app_name), std::move(version)}, {}, false);
}

auto instances_t::start(instances::id_t instance_id) //
    -> result_t
{
    return _impl->do_start_sync(std::move(instance_id), false);
}
auto instances_t::start_once(instances::id_t instance_id) //
    -> result_t
{
    return _impl->do_start_sync(std::move(instance_id), true);
}

auto instances_t::stop(instances::id_t instance_id) //
    -> result_t
{
    return _impl->do_stop_sync(std::move(instance_id), false);
}
auto instances_t::stop_once(instances::id_t instance_id) //
    -> result_t
{
    return _impl->do_stop_sync(std::move(instance_id), true);
}

auto instances_t::remove(instances::id_t instance_id) //
    -> result_t
{
    return _impl->do_remove_sync(std::move(instance_id));
}

auto instances_t::export_to(instances::id_t instance_id, fs::path base_path) const //
    -> result_t
{
    return _impl->do_export_to_sync(std::move(instance_id), std::move(base_path));
}

auto instances_t::import_from(instances::instance_t instance, fs::path base_path) //
    -> result_t
{
    return _impl->do_import_from_sync(std::move(instance), std::move(base_path));
}

} // namespace module
} // namespace flecs
