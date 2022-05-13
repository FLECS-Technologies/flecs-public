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
#include <nlohmann/json.hpp>

#include "private/app_manager_private.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_list_apps(nlohmann::json& response)
{
    response["appList"] = nlohmann::json::array();

    const auto apps = _app_db.all_apps();
    for (const auto& app : apps)
    {
        auto json_app = nlohmann::json{};
        json_app["app"] = app.app.c_str();
        json_app["version"] = app.version.c_str();
        json_app["status"] = app_status_to_string(app.status);
        json_app["desired"] = app_status_to_string(app.desired);
        json_app["installedSize"] = app.installed_size;
        json_app["instances"] = nlohmann::json::array();
        const auto instances = _app_db.instances(app.app, app.version);
        for (const auto& instance : instances)
        {
            auto json_instance = nlohmann::json{};
            json_instance["instanceId"] = instance.id;
            json_instance["instanceName"] = instance.description;
            if (instance.status == instance_status_e::CREATED)
            {
                json_instance["status"] = instance_status_to_string(
                    is_instance_running(instance.id) ? instance_status_e::RUNNING : instance_status_e::STOPPED);
            }
            else
            {
                json_instance["status"] = instance_status_to_string(instance.status);
            }
            json_instance["desired"] = instance_status_to_string(instance.desired);
            json_instance["version"] = instance.version;
            json_app["instances"].push_back(json_instance);
        }
        response["appList"].push_back(json_app);
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS