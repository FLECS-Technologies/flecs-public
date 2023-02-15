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

#include "instances.h"

#include "common/app/app.h"
#include "common/instance/instance.h"
#include "factory/factory.h"
#include "impl/instances_impl.h"

namespace FLECS {

namespace {
register_module_t<module_instances_t> _reg("instances");
}

module_instances_t::module_instances_t()
    : _impl{new impl::module_instances_t{this}}
{}

module_instances_t::~module_instances_t()
{}

auto module_instances_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/instances").methods("GET"_method)([this](const crow::request& req) {
        auto app = req.url_params.get("app");
        auto version = req.url_params.get("version");
        return http_list(app_key_t{app ? app : "", version ? version : ""});
    });

    FLECS_V2_ROUTE("/instances/<string>")
        .methods("GET"_method)([this](const std::string& instance_id) {
            return http_details(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/create").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON_VALUE(args, appKey, app_key_t);
        OPTIONAL_JSON_VALUE(args, instanceName);
        return http_create(std::move(appKey), std::move(instanceName));
    });

    FLECS_V2_ROUTE("/instances/<string>")
        .methods("DELETE"_method)([this](const std::string& instance_id) {
            return http_remove(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/<string>/start")
        .methods("POST"_method)([this](const std::string& instance_id) {
            return http_start(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/<string>/stop")
        .methods("POST"_method)([this](const std::string& instance_id) {
            return http_stop(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/<string>/config")
        .methods("GET"_method)([this](const std::string& instance_id) {
            return http_get_config(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/<string>/config")
        .methods("POST"_method)([this](const crow::request& req, const std::string& instance_id) {
            auto response = json_t{};
            const auto args = parse_json(req.body);
            auto config = instance_config_t{};
            if (args.contains("networkAdapters")) {
                args["networkAdapters"].get_to(config.networkAdapters);
            }
            if (args.contains("devices") && args["devices"].contains("usb")) {
                args["devices"]["usb"].get_to(config.usb_devices);
            }

            return http_post_config(instance_id_t{instance_id}, config);
        });

    FLECS_V2_ROUTE("/instances/<string>/logs")
        .methods("GET"_method)([this](const std::string& instance_id) {
            return http_logs(instance_id_t{instance_id});
        });

    FLECS_V2_ROUTE("/instances/<string>/export")
        .methods("POST"_method)([this](const std::string& instance_id) {
            return http_export_to(instance_id_t{instance_id}, "/var/lib/flecs/exports/");
        });

    return _impl->do_init();
}

auto module_instances_t::http_list(const app_key_t& app_key) const //
    -> crow::response
{
    auto response = json_t::array();

    for (const auto& instance_id : instance_ids(app_key)) {
        auto json = json_t::object();

        auto instance = query(instance_id);

        json["instanceId"] = instance->id().hex();
        json["instanceName"] = instance->instance_name();
        if (auto app = instance->app()) {
            json["appKey"] = app->key();
            if (instance->status() == instance_status_e::Created) {
                json["status"] = to_string(
                    is_running(instance) ? instance_status_e::Running : instance_status_e::Stopped);
            } else {
                json["status"] = to_string(instance->status());
            }
        } else {
            json["appKey"] = app_key_t{instance->app_name().data(), instance->app_version().data()};
            json["status"] = to_string(instance_status_e::Orphaned);
        }
        json["desired"] = to_string(instance->desired());
        response.push_back(std::move(json));
    }

    return {crow::status::OK, "json", response.dump()};
}

auto module_instances_t::http_details(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_details(std::move(instance_id));
}

auto module_instances_t::http_create(app_key_t app_key, std::string instance_name) //
    -> crow::response
{
    auto job_id = _impl->queue_create(std::move(app_key), std::move(instance_name));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::http_start(instance_id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_start(std::move(instance_id));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::http_stop(instance_id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_stop(std::move(instance_id));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::http_remove(instance_id_t instance_id) //
    -> crow::response
{
    auto job_id = _impl->queue_remove(std::move(instance_id));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::http_get_config(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_get_config(std::move(instance_id));
}

auto module_instances_t::http_post_config(
    instance_id_t instance_id, const instance_config_t& config) //
    -> crow::response
{
    return _impl->do_post_config(std::move(instance_id), config);
}

auto module_instances_t::http_logs(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_logs(std::move(instance_id));
}

auto module_instances_t::http_update(instance_id_t instance_id, std::string to) //
    -> crow::response
{
    auto job_id = _impl->queue_update(std::move(instance_id), std::move(to));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::http_export_to(instance_id_t instance_id, fs::path dest_dir) const //
    -> crow::response
{
    auto job_id = _impl->queue_export_to(std::move(instance_id), std::move(dest_dir));
    return crow::response{
        crow::status::ACCEPTED,
        "json",
        "{\"jobId\":" + std::to_string(job_id) + "}"};
}

auto module_instances_t::instance_ids(const app_key_t& app_key) const //
    -> std::vector<instance_id_t>
{
    return _impl->do_instance_ids(app_key);
}
auto module_instances_t::instance_ids(std::string app_name, std::string version) const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{std::move(app_name), std::move(version)});
}
auto module_instances_t::instance_ids(std::string app_name) const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{std::move(app_name), {}});
}
auto module_instances_t::instance_ids() const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{});
}

auto module_instances_t::query(instance_id_t instance_id) const //
    -> std::shared_ptr<instance_t>
{
    return _impl->do_query(std::move(instance_id));
}

auto module_instances_t::is_running(std::shared_ptr<instance_t> instance) const //
    -> bool
{
    return _impl->do_is_running(std::move(instance));
}

auto module_instances_t::create(app_key_t app_key, std::string instance_name) //
    -> result_t
{
    return _impl->do_create_sync(std::move(app_key), std::move(instance_name));
}
auto module_instances_t::create(app_key_t app_key) //
    -> result_t
{
    return create(std::move(app_key), {});
}
auto module_instances_t::create(
    std::string app_name, std::string version, std::string instance_name) //
    -> result_t
{
    return create(app_key_t{std::move(app_name), std::move(version)}, std::move(instance_name));
}
auto module_instances_t::create(std::string app_name, std::string version) //
    -> result_t
{
    return create(app_key_t{std::move(app_name), std::move(version)}, {});
}

auto module_instances_t::start(instance_id_t instance_id) //
    -> result_t
{
    return _impl->do_start_sync(std::move(instance_id));
}

auto module_instances_t::stop(instance_id_t instance_id) //
    -> result_t
{
    return _impl->do_stop_sync(std::move(instance_id));
}

auto module_instances_t::remove(instance_id_t instance_id) //
    -> result_t
{
    return _impl->do_remove_sync(std::move(instance_id));
}

} // namespace FLECS
