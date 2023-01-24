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
    const instance_id_t& /*instance_id*/,
    const std::string& /*app_name*/,
    const std::string& /*version*/,
    json_t& /*response*/) //
    -> crow::status
{
#if 0
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = instance_id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] =
            "Could not delete instance " + instance_id.hex() + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    // Step 2: Do some cross-checks if app_name and version are provided
    auto& instance = _deployment->instances().at(instance_id);

    // correct response based on actual instance
    response["app"] = instance.app_name();
    response["version"] = instance.app_version();

    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0) {
        response["additionalInfo"] = "Could not delete instance: instance/app mismatch";
        return crow::status::BAD_REQUEST;
    }

    // Step 3: Attempt to stop instance
    const auto res = do_stop_instance(instance_id, app_name, version, response, true);

    if (res != crow::status::OK) {
        std::fprintf(
            stderr,
            "Could not stop instance %s: %d\n",
            instance_id.hex().c_str(),
            static_cast<int>(res));
    }

    // Step 4: Remove Docker container for instance
    {
        auto docker_process = process_t{};
        const auto name = std::string{"flecs-"} + instance_id.hex();
        docker_process.spawnp("docker", "rm", "-f", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0) {
            std::fprintf(stderr, "Could not remove docker container %s\n", name.c_str());
        }
    }

    // Step 5: Attempt to load app manifest
    if (!is_app_installed(instance.app_name(), instance.app_version())) {
        std::fprintf(
            stderr,
            "Could not remove volumes of app %s (%s): manifest error\n",
            instance.app_name().c_str(),
            instance.app_version().c_str());
    } else {
        // Step 6: Remove volumes of instance, if manifest loaded successfully
        _deployment->delete_volumes(instance);
    }

    // @todo: move functionality to deployment
    _deployment->delete_instance(instance_id);
    _deployment->save();

    // Final step: Persist removal of instance into db
    persist_apps();
#endif // 0
    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
