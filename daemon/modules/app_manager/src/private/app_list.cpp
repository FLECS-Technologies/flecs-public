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

#include "private/app_manager_private.h"
#include "util/json/json.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_list_apps(json_t& response)
{
    response["appList"] = json_t::array();

    for (decltype(auto) app : _installed_apps)
    {
        auto j = json_t{};
        to_json(j, app.second);
        j["instances"] = json_t::array();
        const auto instances = _app_db.instances(app.second.app(), app.second.version());
        for (const auto& instance : instances)
        {
            auto json_instance = json_t{};
            json_instance["instanceId"] = instance.id;
            json_instance["instanceName"] = instance.description;
            if (instance.status == instance_status_e::CREATED)
            {
                json_instance["status"] = to_string(
                    is_instance_running(instance.id) ? instance_status_e::RUNNING : instance_status_e::STOPPED);
            }
            else
            {
                json_instance["status"] = to_string(instance.status);
            }
            json_instance["desired"] = to_string(instance.desired);
            json_instance["version"] = instance.version;
            j["instances"].push_back(json_instance);
        }
        response["appList"].push_back(j);
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS