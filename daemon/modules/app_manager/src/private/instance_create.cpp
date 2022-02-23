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
#include <limits>
#include <random>

#include "app/app.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_create_instance(
    const std::string& app_name, const std::string& version, const std::string& description, Json::Value& response)
{
    auto status = instance_status_e::NOT_CREATED;
    const auto desired = instance_status_e::CREATED;

    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version))
    {
        response["additionalInfo"] = "Could not create instance of " + app_name + " (" + version + "): not installed";
        return http_status_e::BadRequest;
    }

    // Step 2: Load app manifest
    const auto path = build_manifest_path(app_name, version);
    app_t app{path};
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open manifest " + path;
        return http_status_e::InternalServerError;
    }

    // Step 3: Ensure there is only one instance of single-instance apps
    if (!app.multi_instance())
    {
        decltype(auto) instances = _app_db.instances(app.name(), app.version());
        if (instances.size() > 1)
        {
            std::fprintf(
                stderr,
                "Warning: Multiple instances found for single-instance app %s (%s). Please consider uninstalling and "
                "reinstalling the app.\n",
                app.name().c_str(),
                app.version().c_str());
        }
        if (instances.size() > 0)
        {
            decltype(auto) instance = instances[0];
            response["instanceId"] = instance.id;
            return http_status_e::Ok;
        }
    }

    // Step 4: Create unique id for this instance
    auto seed = std::random_device{};
    auto generator = std::mt19937{seed()};
    auto distribution = std::uniform_int_distribution{
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()};

    auto id = distribution(generator);
    auto hex_id = std::string(8, '\0');
    std::snprintf(hex_id.data(), hex_id.length() + 1, "%.8x", id);
    // Repeat in the unlikely case that the id already exists
    while (_app_db.has_instance({hex_id}))
    {
        id = distribution(generator);
        std::snprintf(hex_id.data(), hex_id.length() + 1, "%.8x", id);
    }

    status = instance_status_e::REQUESTED;

    _app_db.insert_instance({hex_id, app.name(), app.version(), description, status, desired, 0});

    // Step 5: Create Docker volumes
    for (const auto& volume : app.volumes())
    {
        auto docker_process = process_t{};
        const auto name = std::string{"flecs-"} + hex_id + "-" + volume.first;
        docker_process.spawnp("docker", "volume", "create", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            response["additionalInfo"] = docker_process.stderr();
            return http_status_e::InternalServerError;
        }
    }

    // Step 6: Create required Docker networks, if not exist
    for (const auto& network : app.networks())
    {
        auto docker_inspect_process = process_t{};
        docker_inspect_process.spawnp("docker", "network", "inspect", network);
        docker_inspect_process.wait(false, false);
        if (docker_inspect_process.exit_code() != 0)
        {
            auto docker_create_process = process_t{};
            docker_create_process.spawnp("docker", "network", "create", network);
            docker_create_process.wait(false, true);
            if (docker_create_process.exit_code() != 0)
            {
                response["additionalInfo"] = docker_create_process.stderr();
                return http_status_e::InternalServerError;
            }
        }
    }

    status = instance_status_e::RESOURCES_READY;
    _app_db.insert_instance({hex_id, app.name(), app.version(), description, status, desired, 0});

    // Step 7: Create Docker container
    auto docker_process = process_t{};
    docker_process.arg("create");
    for (const auto& volume : app.volumes())
    {
        docker_process.arg("--volume");
        docker_process.arg("flecs-" + hex_id + "-" + volume.first + ":" + volume.second);
    }
    for (const auto& bind_mount : app.bind_mounts())
    {
        docker_process.arg("--volume");
        docker_process.arg(bind_mount.first + ":" + bind_mount.second);
    }

    for (const auto& network : app.networks())
    {
        docker_process.arg("--network");
        docker_process.arg(network);
    }
    for (const auto& port : app.ports())
    {
        docker_process.arg("--publish");
        docker_process.arg(std::to_string(port.first) + ":" + std::to_string(port.second));
    }

    if (app.interactive())
    {
        docker_process.arg("--interactive");
    }

    docker_process.arg("--name");
    docker_process.arg("flecs-" + hex_id);
    docker_process.arg(app.image_with_tag());

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        response["additionalInfo"] = docker_process.stderr();
        return http_status_e::InternalServerError;
    }

    status = instance_status_e::CREATED;

    // Final step: Persist successful creation into db

    _app_db.insert_instance({hex_id, app.name(), app.version(), description, status, desired, 0});
    _app_db.persist();

    response["instanceId"] = hex_id;

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS