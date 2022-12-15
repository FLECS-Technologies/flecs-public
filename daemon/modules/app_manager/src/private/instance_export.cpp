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

auto module_app_manager_private_t::do_export_instance(const std::string& instance_id) //
    -> crow::response
{
    auto response = json_t{};

    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = std::string{};
    response["instanceId"] = instance_id;
    response["version"] = std::string{};

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] = "Could not export instance " + instance_id + ", which does not exist";
        return {crow::status::BAD_REQUEST, response.dump()};
    }

    // get instance details from database
    auto& instance = _deployment->instances().at(instance_id);
    // complete response based on actual instance
    response["app"] = instance.app_name();
    response["version"] = instance.app_version();

    // Step 2: Forward to deployment
    const auto [res, additional_info] = _deployment->export_instance(instance, "/var/lib/flecs/export-tmp/instances/");
    response["additionalInfo"] = additional_info;

    return {(res == 0) ? crow::status::OK : crow::status::INTERNAL_SERVER_ERROR, response.dump()};
}

} // namespace Private
} // namespace FLECS
