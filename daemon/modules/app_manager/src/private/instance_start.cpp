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
    const std::string& instance_id,
    const std::string& app_name,
    const std::string& version,
    json_t& response,
    bool internal) //
    -> crow::status
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = instance_id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist and is fully created
    if (!_app_db.has_instance({instance_id}))
    {
        response["additionalInfo"] = "Could not start instance " + instance_id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    if (!is_instance_runnable(instance_id))
    {
        response["additionalInfo"] = "Could not start instance " + instance_id + ", which is not fully created";
        return crow::status::BAD_REQUEST;
    }

    // get instance details from database
    auto instance = _app_db.query_instance({instance_id}).value();
    // correct response based on actual instance
    response["app"] = instance.app;
    response["version"] = instance.version;

    // Step 2: Do some cross-checks if app_name and version are provided
    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0)
    {
        response["additionalInfo"] = "Could not start instance: instance/app mismatch";
        return crow::status::BAD_REQUEST;
    }

    // Step 3: Return if instance is already running
    if (is_instance_running(instance_id))
    {
        response["additionalInfo"] = "Instance " + instance_id + " already running";
        return crow::status::OK;
    }

    // Step 3: Persist desired status into db, if triggered externally
    if (!internal)
    {
        instance.desired = instance_status_e::RUNNING;
        _app_db.insert_instance(instance);
        _app_db.persist();
    }

    // Final step: Forward to deployment
    const auto& app = _installed_apps.at(std::forward_as_tuple(instance.app, instance.version));
    const auto [res, additional_info] = _deployment->start_instance(app, instance_id);

    response["additionalInfo"] = additional_info;
    return (res == 0) ? crow::status::OK : crow::status::INTERNAL_SERVER_ERROR;
}

} // namespace Private
} // namespace FLECS