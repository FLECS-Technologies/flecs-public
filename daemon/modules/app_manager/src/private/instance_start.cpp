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

auto module_app_manager_private_t::do_start_instance(
    const instance_id_t& /*instance_id*/,
    const std::string& /*app_name*/,
    const std::string& /*version*/,
    json_t& /*response*/,
    bool /*internal*/) //
    -> crow::status
{
#if 0
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = instance_id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist and is fully created
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] =
            "Could not start instance " + instance_id.hex() + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    if (!_deployment->is_instance_runnable(instance_id)) {
        response["additionalInfo"] =
            "Could not start instance " + instance_id.hex() + ", which is not fully created";
        return crow::status::BAD_REQUEST;
    }

    // get instance details from deployment
    auto& instance = _deployment->instances().at(instance_id);
    // correct response based on actual instance
    response["app"] = instance.app_name();
    response["version"] = instance.app_version();

    // Step 2: Do some cross-checks if app_name and version are provided
    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0) {
        response["additionalInfo"] = "Could not start instance: instance/app mismatch";
        return crow::status::BAD_REQUEST;
    }

    // Step 3: Return if instance is already running
    if (_deployment->is_instance_running(instance_id)) {
        response["additionalInfo"] = "Instance " + instance_id.hex() + " already running";
        return crow::status::OK;
    }

    // Step 3: Persist desired status into deployment, if triggered externally
    if (!internal) {
        instance.desired(instance_status_e::Running);
    }

    // Step 4: Forward to deployment
    const auto [res, additional_info] = _deployment->start_instance(instance_id);

    response["additionalInfo"] = additional_info;

    // Final step: Persist instance status into deployment
    _deployment->save();
#endif // 0
    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
