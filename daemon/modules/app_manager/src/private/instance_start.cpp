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
    -> http_status_e
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
        return http_status_e::BadRequest;
    }

    if (!is_instance_runnable(instance_id))
    {
        response["additionalInfo"] = "Could not start instance " + instance_id + ", which is not fully created";
        return http_status_e::BadRequest;
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
        return http_status_e::BadRequest;
    }

    // Step 3: Return if instance is already running
    if (is_instance_running(instance_id))
    {
        response["additionalInfo"] = "Instance " + instance_id + " already running";
        return http_status_e::Ok;
    }

    // Step 3: Persist desired status into db, if triggered externally
    if (!internal)
    {
        instance.desired = instance_status_e::RUNNING;
        _app_db.insert_instance(instance);
        _app_db.persist();
    }

    // Step 4: Load app manifest
    const auto path = build_manifest_path(instance.app, instance.version);
    auto app = app_manifest_t::from_yaml_file(path);
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open manifest " + path.string();
        return http_status_e::InternalServerError;
    }

    // Step 5: Launch app through Docker
    auto docker_process = process_t{};
    const auto name = std::string{"flecs-"} + instance_id;

    docker_process.spawnp("docker", "start", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        response["additionalInfo"] = docker_process.stderr();
        return http_status_e::InternalServerError;
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS