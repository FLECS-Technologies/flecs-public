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
#include <fstream>
#include <limits>
#include <random>

#include "app/manifest/manifest.h"
#include "deployment/deployment_docker.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"
#include "util/string/string_utils.h"

namespace FLECS {
namespace Private {

auto module_app_manager_private_t::do_create_instance(
    const std::string& app_name,
    const std::string& version,
    const std::string& instance_name,
    json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceName"] = instance_name;
    response["version"] = version;

    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version))
    {
        response["additionalInfo"] = "Could not create instance of " + app_name + " (" + version + "): not installed";
        return crow::status::BAD_REQUEST;
    }

    // Step 2: Load app manifest
    const auto& app = _installed_apps.find(app_key_t{app_name, version})->second;

    // Step 3: Ensure there is only one instance of single-instance apps
    if (!app.multi_instance())
    {
        const auto instance_ids = _deployment->instance_ids(app, deployment_t::MatchVersion);
        if (!instance_ids.empty())
        {
            response["instanceId"] = instance_ids[0];
            return crow::status::OK;
        }
    }

    // Step 4: Forward to deployment
    const auto [res, instance_id] = _deployment->create_instance(app, instance_name);

    response["instanceId"] = instance_id;

    // Final step: Persist creation into db
    _deployment->save();

    if (res != 0)
    {
        response["additionalInfo"] = "Failed to create instance";
        return crow::status::INTERNAL_SERVER_ERROR;
    }

    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
