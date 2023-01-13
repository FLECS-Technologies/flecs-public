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

#include "common/app/app_key.h"
#include "common/instance/instance_config.h"
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
        return list(app ? app : "", version ? version : "");
    });

    FLECS_V2_ROUTE("/instances/<string>")
        .methods("GET"_method)(
            [this](const std::string& instance_id) { return details(instance_id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/create").methods("POST"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON_VALUE(args, appKey, app_key_t);
        OPTIONAL_JSON_VALUE(args, instanceName);
        return create(std::move(appKey), std::move(instanceName));
    });

    FLECS_V2_ROUTE("/instances/<string>")
        .methods("DELETE"_method)(
            [this](const std::string& instance_id) { return remove(instance_id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/<string>/start")
        .methods("POST"_method)(
            [this](const std::string& instance_id) { return start(instance_id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/<string>/stop")
        .methods("POST"_method)(
            [this](const std::string& instance_id) { return stop(instance_id_t{instance_id}); });

    FLECS_V2_ROUTE("/instances/<string>/config")
        .methods("GET"_method)([this](const std::string& instance_id) {
            return get_config(instance_id_t{instance_id});
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

            return post_config(instance_id_t{instance_id}, config);
        });

    FLECS_V2_ROUTE("/instances/<string>/logs")
        .methods("GET"_method)(
            [this](const std::string& instance_id) { return logs(instance_id_t{instance_id}); });

    return _impl->do_init();
}

auto module_instances_t::list(const app_key_t& app_key) const //
    -> crow::response
{
    return _impl->do_list(app_key);
}
auto module_instances_t::list(std::string app_name, std::string version) const //
    -> crow::response
{
    return list(app_key_t{std::move(app_name), std::move(version)});
}
auto module_instances_t::list(std::string app_name) const //
    -> crow::response
{
    return list(std::move(app_name), {});
}
auto module_instances_t::list() const //
    -> crow::response
{
    return list({}, {});
}

auto module_instances_t::create(
    app_key_t app_key,
    std::string instance_name) //
    -> crow::response
{
    return _impl->queue_create(std::move(app_key), std::move(instance_name));
}

auto module_instances_t::create(app_key_t app_key) //
    -> crow::response
{
    return create(std::move(app_key), {});
}

auto module_instances_t::create(
    std::string app_name,
    std::string version,
    std::string instance_name) //
    -> crow::response
{
    return create(app_key_t{std::move(app_name), std::move(version)}, std::move(instance_name));
}

auto module_instances_t::create(
    std::string app_name,
    std::string version) //
    -> crow::response
{
    return create(std::move(app_name), std::move(version), {});
}

auto module_instances_t::start(instance_id_t instance_id) //
    -> crow::response
{
    return _impl->queue_start(std::move(instance_id));
}

auto module_instances_t::stop(instance_id_t instance_id) //
    -> crow::response
{
    return _impl->queue_stop(std::move(instance_id));
}

auto module_instances_t::remove(instance_id_t instance_id) //
    -> crow::response
{
    return _impl->queue_remove(std::move(instance_id));
}

auto module_instances_t::get_config(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_get_config(std::move(instance_id));
}

auto module_instances_t::post_config(instance_id_t instance_id, const instance_config_t& config) //
    -> crow::response
{
    return _impl->do_post_config(std::move(instance_id), config);
}

auto module_instances_t::details(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_details(std::move(instance_id));
}

auto module_instances_t::logs(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->do_logs(std::move(instance_id));
}

auto module_instances_t::update(instance_id_t instance_id, std::string from, std::string to) //
    -> crow::response
{
    return _impl->queue_update(std::move(instance_id), std::move(from), std::move(to));
}

auto module_instances_t::archive(instance_id_t instance_id) const //
    -> crow::response
{
    return _impl->queue_export(std::move(instance_id));
}

} // namespace FLECS
