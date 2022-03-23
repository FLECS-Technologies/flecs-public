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

http_status_e module_app_manager_private_t::do_stop_instance(
    const std::string& id, const std::string& app_name, const std::string& version, Json::Value& response,
    bool internal)
{
    // Provisional response based on request
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["instanceId"] = id;
    response["version"] = version;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        response["additionalInfo"] = "Could not stop instance " + id + ", which does not exist";
        return http_status_e::BadRequest;
    }

    // Step 1a: Persist status into db
    // Previous beta versions kept the actual status in the database, which now changed to determining it from
    // Docker directly. Therefore, only the desired status is updated while the actual status remains in its original
    // state (i.e. "CREATED" for runnable instances)
    /** @todo remove for release */
    auto instance = _app_db.query_instance({id}).value();
    if (instance.status == instance_status_e::RUNNING)
    {
        instance.status = instance_status_e::CREATED;
        _app_db.insert_instance(instance);
        _app_db.persist();
    }

    // correct response based on actual instance
    response["app"] = instance.app;
    response["version"] = instance.version;

    // Step 2: Do some cross-checks if app_name and version are provided
    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck < 0)
    {
        response["additionalInfo"] = "Could not stop instance: instance/app mismatch";
        return http_status_e::BadRequest;
    }

    // Step 3: Return if instance is not running
    if (!is_instance_running(id) && !internal)
    {
        response["additionalInfo"] = "Instance " + id + " is not running";
        return http_status_e::Ok;
    }

    // Step 4: Persist desired status into db, if triggered externally
    if (!internal)
    {
        instance.desired = instance_status_e::STOPPED;
        _app_db.insert_instance(instance);
        _app_db.persist();
    }

    // Step 5: Stop instance through Docker
    auto docker_process = process_t{};
    const auto name = std::string{"flecs-"} + id;
    docker_process.spawnp("docker", "stop", name);
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