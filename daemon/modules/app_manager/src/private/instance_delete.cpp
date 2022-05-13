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

http_status_e module_app_manager_private_t::do_delete_instance(
    const std::string& id, const std::string& app_name, const std::string& version, nlohmann::json& response)
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        response["additionalInfo"] = "Could not delete instance " + id + ", which does not exist";
        return http_status_e::BadRequest;
    }

    // Step 2: Do some cross-checks if app_name and version are provided
    auto instance = _app_db.query_instance({id}).value();

    // correct response based on actual instance
    response["app"] = instance.app;
    response["version"] = instance.version;

    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0)
    {
        response["additionalInfo"] = "Could not delete instance: instance/app mismatch";
        return http_status_e::BadRequest;
    }

    // Step 3: Attempt to stop instance
    const auto res = do_stop_instance(id, app_name, version, response, true);
    if (res != http_status_e::Ok)
    {
        std::fprintf(stderr, "Could not stop instance %s: %d\n", id.c_str(), static_cast<int>(res));
    }

    // Step 4: Remove Docker container for instance
    {
        auto docker_process = process_t{};
        const auto name = std::string{"flecs-"} + id;
        docker_process.spawnp("docker", "rm", "-f", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            std::fprintf(stderr, "Could not remove docker container %s\n", name.c_str());
        }
    }

    // Step 5: Attempt to load app manifest
    const auto path = build_manifest_path(instance.app, instance.version);
    auto app = app_t::from_file(path);
    if (!app.yaml_loaded())
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
        for (const auto& volume : app.volumes())
        {
            auto docker_process = process_t{};
            const auto name = std::string{"flecs-"} + id + "-" + volume.first;
            docker_process.spawnp("docker", "volume", "rm", name);
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0)
            {
                std::fprintf(stderr, "Could not remove docker volume %s\n", name.c_str());
            }
        }
    }

    // Final step: Persist removal of instance into db
    _app_db.delete_instance({id});
    _app_db.persist();

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS