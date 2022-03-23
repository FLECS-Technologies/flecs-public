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

#include <cpr/cpr.h>

#include "app/app.h"
#include "private/app_manager_private.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

namespace {

std::string build_manifest_url(const std::string& app_name, const std::string& version)
{
#ifndef NDEBUG
    auto url = std::string{"https://marketplace.flecs.tech:8443/manifests/apps/"};
#else
    auto url = std::string{"https://marketplace.flecs.tech/manifests/apps/"};
#endif // NDEBUG

    url.append(app_name);
    url.append("/");
    url.append(version);
    url.append("/");
    url.append("manifest.yml");

    return url;
}

int download_manifest(const std::string& app_name, const std::string& version)
{
    const auto path = build_manifest_path(app_name, version);
    const auto manifest = fopen(path.c_str(), "w");
    if (manifest == nullptr)
    {
        std::fprintf(stderr, "Could not open %s for writing\n", path.c_str());
        return -1;
    }

    const auto url = build_manifest_url(app_name, version);
    auto response = cpr::Get(cpr::Url{url.c_str()});
    if (response.status_code != static_cast<long>(http_status_e::Ok))
    {
        std::fprintf(stderr, "Could not download app manifest: HTTP return code %ld\n", response.status_code);
        return -1;
    }
    const auto bytes_written = fwrite(response.text.data(), 1, response.text.length(), manifest);
    fclose(manifest);
    if (bytes_written != response.text.length())
    {
        std::fprintf(stderr, "Could not download app manifest: Write error %d\n", errno);
        return -1;
    }

    return 0;
}

} // namespace

http_status_e module_app_manager_private_t::do_install(
    const std::string& app_name, const std::string& version, Json::Value& response)
{
    response["app"] = app_name;
    response["version"] = version;

    // Download app manifest and forward to manifest installation, if download successful
    const auto res = download_manifest(app_name, version);
    if (res != 0)
    {
        response["additionalInfo"] = "Could not download manifest for " + app_name + " (" + version + ")";
        return http_status_e::InternalServerError;
    };

    return do_install(build_manifest_path(app_name, version), response);
}

http_status_e module_app_manager_private_t::do_install(const std::string& manifest, Json::Value& response)
{
    const auto desired = INSTALLED;

    // Step 1: Load app manifest
    auto app = app_t{manifest};
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open app manifest " + manifest;
        return http_status_e::InternalServerError;
    }
    response["additionalInfo"] = std::string{};
    response["app"] = app.name();
    response["version"] = app.version();

    // Step 2: Add database entry for app. At this point the existence of the requested app is guaranteed as its
    // manifest was transferred to the local storage, so it is safe to add it to the local app database
    auto status = MANIFEST_DOWNLOADED;
    _app_db.insert_app({app.name(), app.version(), status, desired, app.category(), 0});

    // Step 3: Pull Docker image for this app
    auto docker_process = process_t{};
    docker_process.spawnp("docker", "pull", app.image_with_tag());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        response["additionalInfo"] = docker_process.stderr();
        return http_status_e::InternalServerError;
    }

    // Placeholder for future extensions. As of now, the installation is complete once the image is downloaded
    // status = IMAGE_DOWNLOADED;

    status = INSTALLED;

    // Final step: Persist successful installation into db
    _app_db.insert_app(apps_table_entry_t{app.name(), app.version(), status, desired, app.category(), 0});
    _app_db.persist();

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
