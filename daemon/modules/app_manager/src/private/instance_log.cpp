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

http_status_e module_app_manager_private_t::do_instance_log(const std::string& id, nlohmann::json& response)
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

    // Step 2: Obtain log from Docker
    auto docker_process = process_t{};
    docker_process.spawnp("docker", "logs", "flecs-" + id);
    docker_process.wait(false, false);

    // Step 3: Build response
    if (docker_process.exit_code() != 0)
    {
        response["additionalInfo"] = "Could not get logs for instance " + id;
        return http_status_e::InternalServerError;
    }

    response["log"] = "--- stdout\n" + docker_process.stdout() + "\n--- stderr\n" + docker_process.stderr();

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
