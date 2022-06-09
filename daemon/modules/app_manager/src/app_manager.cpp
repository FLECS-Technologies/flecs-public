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
{
    using namespace std::placeholders;

    api::register_endpoint("/app/install", HTTP_POST, std::bind(&module_app_manager_t::install, this, _1, _2));
    api::register_endpoint("/app/instances", HTTP_POST, std::bind(&module_app_manager_t::list_instances, this, _1, _2));
    api::register_endpoint("/app/list", HTTP_GET, std::bind(&module_app_manager_t::list_apps, this, _1, _2));
    api::register_endpoint("/app/sideload", HTTP_PUT, std::bind(&module_app_manager_t::sideload, this, _1, _2));
    api::register_endpoint("/app/uninstall", HTTP_POST, std::bind(&module_app_manager_t::uninstall, this, _1, _2));
    api::register_endpoint("/app/versions", HTTP_POST, std::bind(&module_app_manager_t::list_versions, this, _1, _2));
    api::register_endpoint(
        "/instance/config",
        HTTP_POST,
        std::bind(&module_app_manager_t::post_config_instance, this, _1, _2));
    api::register_endpoint(
        "/instance/config",
        HTTP_PUT,
        std::bind(&module_app_manager_t::put_config_instance, this, _1, _2));
    api::register_endpoint(
        "/instance/create",
        HTTP_POST,
        std::bind(&module_app_manager_t::create_instance, this, _1, _2));
    api::register_endpoint(
        "/instance/delete",
        HTTP_POST,
        std::bind(&module_app_manager_t::delete_instance, this, _1, _2));
    api::register_endpoint(
        "/instance/start",
        HTTP_POST,
        std::bind(&module_app_manager_t::start_instance, this, _1, _2));
    api::register_endpoint("/instance/stop", HTTP_POST, std::bind(&module_app_manager_t::stop_instance, this, _1, _2));
    api::register_endpoint(
        "/instance/details",
        HTTP_POST,
        std::bind(&module_app_manager_t::instance_details, this, _1, _2));
    api::register_endpoint("/instance/log", HTTP_POST, std::bind(&module_app_manager_t::instance_log, this, _1, _2));
}

module_app_manager_t::~module_app_manager_t()
{}

auto module_app_manager_t::do_init() //
    -> void
{
    return _impl->do_init();
}

auto module_app_manager_t::install(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    OPTIONAL_JSON_VALUE(args, licenseKey);
    return _impl->do_install(app, version, licenseKey, response);
}

auto module_app_manager_t::sideload(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, appYaml);
    OPTIONAL_JSON_VALUE(args, licenseKey);
    return _impl->do_sideload(appYaml, licenseKey, response);
}

auto module_app_manager_t::uninstall(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    return _impl->do_uninstall(app, version, response);
}

auto module_app_manager_t::create_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    OPTIONAL_JSON_VALUE(args, instanceName);
    return _impl->do_create_instance(app, version, instanceName, response);
}

auto module_app_manager_t::delete_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_delete_instance(instanceId, app, version, response);
}

auto module_app_manager_t::start_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_start_instance(instanceId, app, version, response);
}

auto module_app_manager_t::stop_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_stop_instance(instanceId, app, version, response);
}

auto module_app_manager_t::instance_details(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    return _impl->do_instance_details(instanceId, response);
}

auto module_app_manager_t::instance_log(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    return _impl->do_instance_log(instanceId, response);
}

auto module_app_manager_t::list_apps(const json_t& /*args*/, json_t& response) //
    -> http_status_e
{
    return _impl->do_list_apps(response);
}

auto module_app_manager_t::list_versions(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, app_name);
    return _impl->do_list_versions(app_name, response);
}

auto module_app_manager_t::list_instances(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, app_name);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_list_instances(app_name, version, response);
}

auto module_app_manager_t::post_config_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    return _impl->do_post_config_instance(instanceId, response);
}

auto module_app_manager_t::put_config_instance(const json_t& args, json_t& response) //
    -> http_status_e
{
    REQUIRED_JSON_VALUE(args, instanceId);
    auto config = instance_config_t{};
    for (decltype(auto) it : args["networkAdapters"])
    {
        REQUIRED_TYPED_JSON_VALUE(it, name, std::string);
        REQUIRED_TYPED_JSON_VALUE(it, active, bool);
        OPTIONAL_JSON_VALUE(it, ipAddress);
        OPTIONAL_JSON_VALUE(it, subnetMask);
        OPTIONAL_JSON_VALUE(it, gateway);
        config.networkAdapters.push_back({name, ipAddress, subnetMask, gateway, active});
    }

    return _impl->do_put_config_instance(instanceId, config, response);
}

} // namespace FLECS
