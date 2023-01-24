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
#include "util/json/json.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

auto module_app_manager_private_t::do_export_app(const std::string& /*app_name*/, const std::string& /*version*/) //
    -> crow::response
{
#if 0
    auto response = json_t{};

    response["additionalInfo"] = std::string{};
    response["app"] = app_name;
    response["version"] = version;

    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version)) {
        response["additionalInfo"] = "Could not export " + app_name + " (" + version + "): not installed";
        return {crow::status::BAD_REQUEST, response.dump()};
    }

    // Step 2: Load app manifest
    auto& app = _installed_apps.find(app_key_t{app_name, version})->second;

    // Step 3: Create export directory
    auto ec = std::error_code{};
    fs::create_directories("/var/lib/flecs/export-tmp/apps/");
    if (ec) {
        response["additionalInfo"] = "Could not create app export directory";
        return {crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }

    // Step 4: Export image
    auto docker_process = process_t{};
    const auto filename = std::string{"/var/lib/flecs/export-tmp/apps/"}
                              .append(app.app())
                              .append("_")
                              .append(app.version())
                              .append(".tar");
    docker_process.spawnp("docker", "save", "--output", filename, app.image_with_tag());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        response["additionalInfo"] = docker_process.stderr();
        return {crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }

    // Step 5: Copy manifest
    const auto manifest_src = fs::path{"/var/lib/flecs/apps"} / app_name / version / "manifest.yml";
    const auto manifest_dst = fs::path{"/var/lib/flecs/export-tmp/apps/"} / (app_name + "_" + version + ".yml");
    fs::copy_file(manifest_src, manifest_dst, ec);
    if (ec) {
        response["additionalInfo"] = "Could not copy app manifest";
        return {crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }

#endif // 0
    return {crow::status::OK, "json", "[]"};
}

} // namespace Private
} // namespace FLECS
