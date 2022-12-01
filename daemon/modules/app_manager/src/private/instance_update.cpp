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
#include "util/process/process.h"

namespace FLECS {
namespace Private {

auto module_app_manager_private_t::do_update_instance(
    const std::string& instance_id,
    const std::string& app_name,
    const std::string& from,
    const std::string& to,
    json_t& response) //
    -> crow::status
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = instance_id;
    response["from"] = from;
    response["to"] = to;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] = "Could not update instance " + instance_id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    // get instance details from database
    auto& instance = _deployment->instances().at(instance_id);
    // correct response based on actual instance
    response["app"] = instance.app_name();
    response["from"] = instance.app_version();

    // Step 2: Do some cross-checks if app_name and from-version are provided
    auto xcheck = xcheck_app_instance(instance, app_name, from);
    if (xcheck < 0) {
        response["additionalInfo"] = "Could not update instance: instance/app mismatch";
        return crow::status::BAD_REQUEST;
    }

    // Step 3: Make sure to-app is installed
    if (!is_app_installed(instance.app_name(), to)) {
        response["additionalInfo"] = "Could not update instance to version " + to + ", which is not installed";
        return crow::status::BAD_REQUEST;
    }

    // Step 4: Stop running instance
    const auto stop_res = do_stop_instance(instance.id(), instance.app_name(), instance.app_version(), response, true);
    if (stop_res != crow::status::OK) {
        response["additionalInfo"] = "Could not stop instance " + instance.id();
        return crow::status::INTERNAL_SERVER_ERROR;
    }

    // Step 5: Create backup of existing instance

    // Step 6: Update instance structure
    decltype(auto) app = _installed_apps.at(app_key_t{instance.app_name(), to});
    instance.app(&app);

    // Step 7: Persist updated instance into deployment
    _deployment->save();

    // Final step: Start instance
    if (instance.desired() == instance_status_e::RUNNING) {
        const auto start_res =
            do_start_instance(instance.id(), instance.app_name(), instance.app_version(), response, true);
        if (start_res != crow::status::OK) {
            response["additionalInfo"] = "Could not stop instance " + instance.id();
            return crow::status::INTERNAL_SERVER_ERROR;
        }
    }

    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
