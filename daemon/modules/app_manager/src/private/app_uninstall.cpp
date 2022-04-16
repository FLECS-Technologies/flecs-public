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
#include <filesystem>

#include "app/app.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_uninstall(
    const std::string& app_name, const std::string& version, Json::Value& response)
{
    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["version"] = version;

    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version))
    {
        response["additionalInfo"] = "Could not uninstall " + app_name + " (" + version + "): not installed";
        return http_status_e::BadRequest;
    }

    // Step 2: Load app manifest
    const auto path = build_manifest_path(app_name, version);

    auto app = app_t{path};
    if (!app.yaml_loaded())
    {
        // Manifest missing or invalid - persist removal of app into db
        _app_db.delete_app({app_name, version});
        _app_db.persist();

        response["additionalInfo"] = "Could not open manifest " + path;
        return http_status_e::InternalServerError;
    }

    // Step 2a: Prevent removal of system apps
    if (cxx20::contains(app.category(), "system"))
    {
        response["additionalInfo"] = "Not removing system app " + app.name() + "(" + app.version() + ")";
        return http_status_e::InternalServerError;
    }

    // Step 3: Stop and delete all instances of the app
    const auto instances = _app_db.instances(app_name, version);
    for (auto& instance : instances)
    {
        do_delete_instance(instance.id, app_name, version, response);
    }

    // Step 4: Remove Docker image of the app
    const auto image = app.image_with_tag();
    auto docker_process = process_t{};
    docker_process.spawnp("docker", "rmi", "-f", image);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        std::fprintf(
            stderr,
            "Warning: Could not remove image %s of app %s (%s)\n",
            image.c_str(),
            app_name.c_str(),
            version.c_str());
    }

    // Step 5: Persist removal of app into db
    _app_db.delete_app({app_name, version});
    _app_db.persist();

    // Step 6: Remove app manifest
    auto ec = std::error_code{};
    const auto res = std::filesystem::remove(path, ec);
    if (!res)
    {
        std::fprintf(
            stderr,
            "Warning: Could not remove manifest %s of app %s (%s)\n",
            path.c_str(),
            app_name.c_str(),
            version.c_str());
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
