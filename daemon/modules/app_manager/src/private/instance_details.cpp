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

#include "app/manifest/manifest.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

auto module_app_manager_private_t::do_instance_details(const std::string& id, json_t& response) //
    -> crow::status
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["instanceId"] = id;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        response["additionalInfo"] = "Could not query details of instance " + id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    // Step 2: Obtain instance and corresponsing app
    const auto instance = _app_db.query_instance({id}).value();
    const auto manifest_path = build_manifest_path(instance.app, instance.version);
    const auto app = app_manifest_t::from_yaml_file(manifest_path);

    // Build response
    response["app"] = instance.app;
    response["version"] = instance.version;
    response["IPAddress"] = instance.ips.empty() ? "" : instance.ips[0];
    response["conffiles"] = json_t::array();
    response["hostname"] = app.hostname().empty() ? (std::string{"flecs-"}.append(instance.id)) : app.hostname();
    for (const auto& conffile : app.conffiles())
    {
        auto json_conffile = json_t{};
        json_conffile["host"] = std::string{"/var/lib/flecs/instances/"} + instance.id + "/conf/" + conffile.local();
        json_conffile["container"] = conffile.container();
        response["conffiles"].push_back(json_conffile);
    }
    response["ports"] = json_t::array();
    for (const auto& port : app.ports())
    {
        auto json_port = json_t{};
        json_port["host"] = stringify(port.host_port_range());
        json_port["container"] = stringify(port.container_port_range());
        response["ports"].push_back(json_port);
    }
    response["volumes"] = json_t::array();
    response["mounts"] = json_t::array();
    for (const auto& volume : app.volumes())
    {
        if (volume.type() == volume_t::VOLUME)
        {
            auto json_volume = json_t{};
            json_volume["name"] = volume.host();
            json_volume["path"] = volume.container();
            response["volumes"].push_back(json_volume);
        }
        else if (volume.type() == volume_t::BIND_MOUNT)
        {
            auto json_mount = json_t{};
            json_mount["host"] = volume.host();
            json_mount["container"] = volume.container();
            response["mounts"].push_back(json_mount);
        }
    }

    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
