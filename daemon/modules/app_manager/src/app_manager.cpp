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

#include "factory/factory.h"
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

    api::register_endpoint("/app/install", std::bind(&module_app_manager_t::install, this, _1, _2));
    api::register_endpoint("/app/instances", std::bind(&module_app_manager_t::list_instances, this, _1, _2));
    api::register_endpoint("/app/list", std::bind(&module_app_manager_t::list_apps, this, _1, _2));
    api::register_endpoint("/app/sideload", std::bind(&module_app_manager_t::sideload, this, _1, _2));
    api::register_endpoint("/app/uninstall", std::bind(&module_app_manager_t::uninstall, this, _1, _2));
    api::register_endpoint("/app/versions", std::bind(&module_app_manager_t::list_versions, this, _1, _2));
    api::register_endpoint("/instance/create", std::bind(&module_app_manager_t::create_instance, this, _1, _2));
    api::register_endpoint("/instance/delete", std::bind(&module_app_manager_t::delete_instance, this, _1, _2));
    api::register_endpoint("/instance/start", std::bind(&module_app_manager_t::start_instance, this, _1, _2));
    api::register_endpoint("/instance/stop", std::bind(&module_app_manager_t::stop_instance, this, _1, _2));
    api::register_endpoint("/instance/details", std::bind(&module_app_manager_t::instance_details, this, _1, _2));
    api::register_endpoint("/instance/log", std::bind(&module_app_manager_t::instance_log, this, _1, _2));
}

module_app_manager_t::~module_app_manager_t()
{}

void module_app_manager_t::do_init()
{
    return _impl->do_init();
}

http_status_e module_app_manager_t::install(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    OPTIONAL_JSON_VALUE(args, licenseKey);
    return _impl->do_install(app, version, licenseKey, response);
}

http_status_e module_app_manager_t::sideload(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, appYaml);
    OPTIONAL_JSON_VALUE(args, licenseKey);
    return _impl->do_sideload(appYaml, licenseKey, response);
}

http_status_e module_app_manager_t::uninstall(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    return _impl->do_uninstall(app, version, response);
}

http_status_e module_app_manager_t::create_instance(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, app);
    REQUIRED_JSON_VALUE(args, version);
    OPTIONAL_JSON_VALUE(args, instanceName);
    return _impl->do_create_instance(app, version, instanceName, response);
}

http_status_e module_app_manager_t::delete_instance(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_delete_instance(instanceId, app, version, response);
}

http_status_e module_app_manager_t::start_instance(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_start_instance(instanceId, app, version, response);
}

http_status_e module_app_manager_t::stop_instance(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, instanceId);
    OPTIONAL_JSON_VALUE(args, app);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_stop_instance(instanceId, app, version, response);
}

http_status_e module_app_manager_t::instance_details(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, instanceId);
    return _impl->do_instance_details(instanceId, response);
}

http_status_e module_app_manager_t::instance_log(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, instanceId);
    return _impl->do_instance_log(instanceId, response);
}

http_status_e module_app_manager_t::list_apps(const Json::Value& /*args*/, Json::Value& response)
{
    return _impl->do_list_apps(response);
}

http_status_e module_app_manager_t::list_versions(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, app_name);
    return _impl->do_list_versions(app_name, response);
}

http_status_e module_app_manager_t::list_instances(const Json::Value& args, Json::Value& response)
{
    REQUIRED_JSON_VALUE(args, app_name);
    OPTIONAL_JSON_VALUE(args, version);
    return _impl->do_list_instances(app_name, version, response);
}

} // namespace FLECS
