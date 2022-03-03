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

http_status_e module_app_manager_private_t::do_start_instance(
    const std::string& id, const std::string& app_name, const std::string& version, Json::Value& response,
    bool internal)
{
    response["instanceId"] = id;
    response["app"] = app_name;
    response["version"] = version;
    response["additionalInfo"] = "";
    // Step 1: Verify instance does actually exist and is fully created
    if (!_app_db.has_instance({id}))
    {
        response["additionalInfo"] = "Could not start instance " + id + ", which does not exist";
        return http_status_e::BadRequest;
    }

    if (!is_instance_runnable(id))
    {
        response["additionalInfo"] = "Could not start instance " + id + ", which is not fully created";
        return http_status_e::BadRequest;
    }

    // Step 1a: Persist status into db
    // Previous beta versions kept the actual status in the database, which now changed to determining it from
    // Docker directly. Therefore, only the desired status is updated while the actual status remains in its original
    // state (i.e. "CREATED" for runnable instances)
    auto instance = _app_db.query_instance({id}).value();
    instance.status = instance_status_e::CREATED;
    _app_db.insert_instance(instance);
    _app_db.persist();

    // Step 2: Do some cross-checks if app_name and version are provided
    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0)
    {
        response["additionalInfo"] = "Could not start instance: instance/app mismatch";
        return http_status_e::BadRequest;
    }

    // Step 3: Return if instance is already running
    if (is_instance_running(id))
    {
        response["additionalInfo"] = "Instance " + id + " already running";
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
    app_t app{path};
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open manifest " + path;
        return http_status_e::InternalServerError;
    }

    // Step 5: Launch app through Docker
    auto docker_process = process_t{};
    const auto name = std::string{"flecs-"} + id;

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