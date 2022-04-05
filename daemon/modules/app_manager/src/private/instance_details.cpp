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

#include <cstdio>

#include "app/app.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_instance_details(const std::string& id, Json::Value& response)
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["instanceId"] = id;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        response["additionalInfo"] = "Could not query details of instance " + id + ", which does not exist";
        return http_status_e::BadRequest;
    }

    // Step 2: Obtain instance and corresponsing app
    const auto instance = _app_db.query_instance({id}).value();
    const auto manifest_path = build_manifest_path(instance.app, instance.version);
    const auto app = app_t{manifest_path};

    // Build response
    response["app"] = instance.app;
    response["version"] = instance.version;
    response["IPAddress"] = instance.ip;
    response["conffiles"] = Json::Value{Json::arrayValue};
    response["hostname"] = app.hostname().empty() ? (std::string{"flecs-"}.append(instance.id)) : app.hostname();
    for (const auto& conffile : app.conffiles())
    {
        auto json_conffile = Json::Value{};
        json_conffile["host"] = std::string{"/var/lib/flecs/instances/"} + instance.id + "/conf/" + conffile.local();
        json_conffile["container"] = conffile.container();
        response["conffiles"].append(json_conffile);
    }
    response["ports"] = Json::Value{Json::arrayValue};
    for (const auto& port : app.ports())
    {
        auto json_port = Json::Value{};
        json_port["host"] = stringify(port.host_port_range());
        json_port["container"] = stringify(port.container_port_range());
        response["ports"].append(json_port);
    }
    response["volumes"] = Json::Value{Json::arrayValue};
    for (const auto& volume : app.volumes())
    {
        auto json_volume = Json::Value{};
        json_volume["name"] = volume.first;
        json_volume["path"] = volume.second;
        response["volumes"].append(json_volume);
    }
    response["mounts"] = Json::Value{Json::arrayValue};
    for (const auto& bind_mount : app.bind_mounts())
    {
        auto json_mount = Json::Value{};
        json_mount["host"] = bind_mount.first;
        json_mount["container"] = bind_mount.second;
        response["mounts"].append(json_mount);
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
