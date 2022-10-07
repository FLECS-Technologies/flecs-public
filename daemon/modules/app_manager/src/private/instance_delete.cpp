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

auto module_app_manager_private_t::do_delete_instance(
    const std::string& instance_id,
    const std::string& app_name,
    const std::string& version,
    json_t& response) //
    -> crow::status
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = instance_id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id))
    {
        response["additionalInfo"] = "Could not delete instance " + instance_id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    // Step 2: Do some cross-checks if app_name and version are provided
    auto& instance = _deployment->instances().at(instance_id);

    // correct response based on actual instance
    response["app"] = instance.app_name();
    response["version"] = instance.app_version();

    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0)
    {
        response["additionalInfo"] = "Could not delete instance: instance/app mismatch";
        return crow::status::BAD_REQUEST;
    }

    // Step 3: Attempt to stop instance
    const auto res = do_stop_instance(instance_id, app_name, version, response, true);

    if (res != crow::status::OK)
    {
        std::fprintf(stderr, "Could not stop instance %s: %d\n", instance_id.c_str(), static_cast<int>(res));
    }

    // Step 4: Remove Docker container for instance
    {
        auto docker_process = process_t{};
        const auto name = std::string{"flecs-"} + instance_id;
        docker_process.spawnp("docker", "rm", "-f", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            std::fprintf(stderr, "Could not remove docker container %s\n", name.c_str());
        }
    }

    // Step 5: Attempt to load app manifest
    if (!is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Could not remove volumes of app %s (%s): manifest error\n",
            app_name.c_str(),
            version.c_str());
    }
    // Step 6: Remove volumes of instance, if manifest loaded successfully
    else
    {
        const auto& app = _installed_apps.find(app_key_t{app_name, version})->second;
        for (const auto& volume : app.volumes())
        {
            if (volume.type() != volume_t::VOLUME)
            {
                continue;
            }
            auto docker_process = process_t{};
            const auto name = std::string{"flecs-"} + instance_id + "-" + volume.host();
            docker_process.spawnp("docker", "volume", "rm", name);
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0)
            {
                std::fprintf(stderr, "Could not remove docker volume %s\n", name.c_str());
            }
        }
    }

    // @todo: move functionality to deployment
    _deployment->delete_instance(instance_id);

    // Final step: Persist removal of instance into db
    persist_apps();

    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS