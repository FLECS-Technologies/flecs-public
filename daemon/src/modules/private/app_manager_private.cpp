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

#include "modules/private/app_manager_private.h"

#include <sys/stat.h>
#include <unistd.h>

#include <filesystem>
#include <iomanip>
#include <random>
#include <sstream>

#include "app/app.h"
#include "json/json.h"
#include "util/curl_easy_ext/curl_easy_ext.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

module_error_e download_manifest(const std::string& app_name, const std::string& version);
std::string build_manifest_url(const std::string& app_name, const std::string& version);
std::string build_manifest_path(const std::string& app_name, const std::string& version);

module_app_manager_private_t::module_app_manager_private_t()
    : _app_db{}
{
    // Prune all entries of apps that completely failed to install. This usually means that an app with this combination
    // of name and version does not exist anywhere in the universe, and it should never have been inserted into the db
    // in the first place. As this happened nonetheless in earlier versions, the mess is cleaned up here
    for (decltype(auto) app : _app_db.all_apps())
    {
        if (app.status == app_status_e::NOT_INSTALLED && app.desired == app_status_e::INSTALLED)
        {
            _app_db.delete_app({app.app, app.version});
        }
    }

    std::fprintf(stdout, "Starting all app instances...\n");
    for (decltype(auto) instance : _app_db.all_instances())
    {
        if (instance.desired == instance_status_e::RUNNING)
        {
            std::fprintf(stdout, "\t%s\n", instance.id.c_str());
            do_start_instance(instance.id, "", "", true);
        }
    }
}

module_app_manager_private_t::~module_app_manager_private_t()
{
    std::fprintf(stdout, "Stopping all running app instances...\n");
    for (decltype(auto) instance : _app_db.all_instances())
    {
        if (is_instance_running(instance.id))
        {
            std::fprintf(stdout, "\t%s\n", instance.id.c_str());
            do_stop_instance(instance.id, "", "", true);
        }
    }
}

module_error_e module_app_manager_private_t::do_install(const std::string& app_name, const std::string& version)
{
    // Download app manifest and forward to manifest installation, if download successful
    const auto res = download_manifest(app_name, version);
    if (res != FLECS_OK)
    {
        return res;
    }

    return do_install(build_manifest_path(app_name, version));
}

module_error_e module_app_manager_private_t::do_install(const std::string& manifest)
{
    const auto desired = INSTALLED;

    // Step 1: Load app manifest
    auto app = app_t{manifest};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

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
        return FLECS_DOCKER;
    }

    // Placeholder for future extensions. As of now, the installation is complete once the image is downloaded
    // status = IMAGE_DOWNLOADED;

    status = INSTALLED;

    // Final step: Persist successful installation into db
    _app_db.insert_app(apps_table_entry_t{app.name(), app.version(), status, desired, app.category(), 0});
    _app_db.persist();

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_sideload(const std::string& manifest_path)
{
    // Step 1: Parse transferred manifest
    auto app = app_t{manifest_path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    // Step 2: Copy manifest to local storage
    const auto path = build_manifest_path(app.name(), app.version());

    std::error_code ec;
    std::filesystem::remove(path, ec);
    std::filesystem::copy(manifest_path, path, ec);
    if (ec)
    {
        std::fprintf(stderr, "Could not copy manifest to %s: %d\n", path.c_str(), ec.value());
        return FLECS_IO;
    }

    // Step 3: Forward to manifest installation
    return do_install(manifest_path);
}

module_error_e module_app_manager_private_t::do_uninstall(const std::string& app_name, const std::string& version)
{
    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Could not uninstall %s (%s), which is not installed\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    // Step 2: Load app manifest
    const auto path = build_manifest_path(app_name, version);

    auto app = app_t{path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    // Step 3: Stop and delete all instances of the app
    const auto instances = _app_db.instances(app_name, version);
    for (auto& instance : instances)
    {
        const auto res = do_stop_instance(instance.id, app_name, version, true);
        if (res != FLECS_OK)
        {
            std::fprintf(stderr, "Warning: Could not stop instance %s: %d\n", instance.id.c_str(), res);
        }
        _app_db.delete_instance({instance.id});
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

    // Step 5: Remove app manifest
    auto ec = std::error_code{};
    const auto res = std::filesystem::remove(path, ec);
    if (!res)
    {
        std::fprintf(stderr, "Could not delete manifest %s: %d\n", path.c_str(), ec.value());
        return FLECS_IO;
    }

    // Step 6: Persist removal of app into db
    _app_db.delete_app({app_name, version});
    _app_db.persist();

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_create_instance(
    const std::string& app_name, const std::string& version, const std::string& description)
{
    auto status = instance_status_e::NOT_CREATED;
    const auto desired = instance_status_e::CREATED;

    // Step 1: Ensure app is actually installed
    if (!is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Could not create instance of app %s (%s), which is not installed\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    // Step 2: Load app manifest
    const auto path = build_manifest_path(app_name, version);
    app_t app{path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
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
            std::fprintf(stdout, "%s\n", instance.id.c_str());
            return FLECS_OK;
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
            std::fprintf(stderr, "Could not create docker volume %s\n", name.c_str());
            return FLECS_DOCKER;
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
                std::fprintf(stderr, "Could not create Docker network %s\n", network.c_str());
                return FLECS_DOCKER;
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
        std::fprintf(stderr, "Could not create container for instance %s\n", hex_id.c_str());
        return FLECS_DOCKER;
    }

    status = instance_status_e::CREATED;

    // Final step: Persist successful creation into db

    _app_db.insert_instance({hex_id, app.name(), app.version(), description, status, desired, 0});
    _app_db.persist();

    std::fprintf(stdout, "%s", hex_id.c_str());

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_delete_instance(
    const std::string& id, const std::string& app_name, const std::string& version)
{
    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        std::fprintf(
            stderr,
            "Could not delete instance %s of app %s (%s), which does not exist\n",
            id.c_str(),
            app_name.c_str(),
            version.c_str());
        return FLECS_INSTANCE_NOTEXIST;
    }

    // Step 2: Do some cross-checks if app_name and version are provided
    auto instance = _app_db.query_instance({id}).value();
    auto xcheck = xcheck_app_instance(instance, app_name, version);
    if (xcheck != FLECS_OK)
    {
        return xcheck;
    }

    // Step 3: Attempt to stop instance
    const auto res = do_stop_instance(id, app_name, version, true);
    if (res != FLECS_OK)
    {
        std::fprintf(stderr, "Could not stop instance %s: %d\n", id.c_str(), res);
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
    app_t app{path};
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

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_start_instance(
    const std::string& id, const std::string& app_name, const std::string& version, bool internal)
{
    // Step 1: Verify instance does actually exist and is fully created
    if (!_app_db.has_instance({id}))
    {
        std::fprintf(
            stderr,
            "Could not start instance %s of app %s (%s), which does not exist\n",
            id.c_str(),
            app_name.empty() ? "unspecified" : app_name.c_str(),
            version.empty() ? "unspecified" : version.c_str());
        return FLECS_INSTANCE_NOTEXIST;
    }

    if (!is_instance_runnable(id))
    {
        std::fprintf(
            stderr,
            "Could not start instance %s of app %s (%s), which is not fully created\n",
            id.c_str(),
            app_name.empty() ? "unspecified" : app_name.c_str(),
            version.empty() ? "unspecified" : version.c_str());
        return FLECS_INSTANCE_NOTRUNNABLE;
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
    if (xcheck != FLECS_OK)
    {
        return xcheck;
    }

    // Step 3: Return if instance is already running
    if (is_instance_running(id))
    {
        std::fprintf(stdout, "Instance %s is already running\n", id.c_str());
        return FLECS_OK;
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
        return FLECS_YAML;
    }

    // Step 5: Launch app through Docker
    auto docker_process = process_t{};
    const auto name = std::string{"flecs-"} + id;

    docker_process.spawnp("docker", "start", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return FLECS_DOCKER;
    }

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_stop_instance(
    const std::string& id, const std::string& app_name, const std::string& version, bool internal)
{
    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({id}))
    {
        std::fprintf(
            stderr,
            "Could not stop instance %s of app %s (%s), which does not exist\n",
            id.c_str(),
            app_name.empty() ? "unspecified" : app_name.c_str(),
            version.empty() ? "unspecified" : version.c_str());
        return FLECS_INSTANCE_NOTEXIST;
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
    if (xcheck != FLECS_OK)
    {
        return xcheck;
    }

    // Step 3: Return if instance is not running
    if (!is_instance_running(id))
    {
        std::fprintf(stdout, "Instance %s is not running\n", id.c_str());
        return FLECS_OK;
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
        return FLECS_DOCKER;
    }

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_list_apps()
{
    Json::Value json_value;
    json_value["appList"] = Json::Value{Json::arrayValue};

    const auto apps = _app_db.all_apps();
    for (const auto& app : apps)
    {
        auto json_app = Json::Value{};
        json_app["app"] = app.app.c_str();
        json_app["version"] = app.version.c_str();
        json_app["status"] = app_status_to_string(app.status);
        json_app["desired"] = app_status_to_string(app.desired);
        json_app["installedSize"] = app.installed_size;
        json_app["instances"] = Json::Value{Json::arrayValue};
        const auto instances = _app_db.instances(app.app, app.version);
        for (const auto& instance : instances)
        {
            auto json_instance = Json::Value{};
            json_instance["instanceId"] = instance.id;
            json_instance["instanceName"] = instance.description;
            if (instance.status == instance_status_e::CREATED)
            {
                json_instance["status"] = instance_status_to_string(
                    is_instance_running(instance.id) ? instance_status_e::RUNNING : instance_status_e::STOPPED);
            }
            else
            {
                json_instance["status"] = instance_status_to_string(instance.status);
            }
            json_instance["desired"] = instance_status_to_string(instance.desired);
            json_instance["version"] = instance.version;
            json_app["instances"].append(json_instance);
        }
        json_value["appList"].append(json_app);
    }
    fprintf(stdout, "%s", json_value.toStyledString().c_str());

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_list_versions(const std::string& /*app_name*/)
{
    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_list_instances(
    const std::string& /*app_name*/, const std::string& /*version*/)
{
    return FLECS_OK;
}

bool module_app_manager_private_t::is_app_installed(const std::string& app_name, const std::string& version)
{
    return _app_db.has_app({app_name, version}) &&
           (_app_db.query_app({app_name, version}).value().status == app_status_e::INSTALLED);
}

bool module_app_manager_private_t::is_instance_runnable(const std::string& id)
{
    if (!_app_db.has_instance({id}))
    {
        return false;
    }

    const auto instance = _app_db.query_instance({id}).value();
    if (instance.status != instance_status_e::CREATED)
    {
        return false;
    }

    return true;
}

bool module_app_manager_private_t::is_instance_running(const std::string& id)
{
    auto docker_process = process_t{};

    docker_process.spawnp("docker", "ps", "--quiet", "--filter", std::string{"name=flecs-" + id});
    docker_process.wait(false, false);
    // Consider instance running if Docker call was successful and returned a container id
    if (docker_process.exit_code() == 0 && !docker_process.stdout().empty())
    {
        return true;
    }

    return false;
}

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

std::string build_manifest_path(const std::string& app_name, const std::string& version)
{
    auto path = std::string{"/var/lib/flecs/apps"};

    path.append("/" + app_name);
    path.append("/" + version);

    auto ec = std::error_code{};
    std::filesystem::create_directories(path, ec);

    path.append("/manifest.yml");

    return path;
}

module_error_e download_manifest(const std::string& app_name, const std::string& version)
{
    const auto path = build_manifest_path(app_name, version);
    const auto manifest = fopen(path.c_str(), "w");
    if (manifest == nullptr)
    {
        std::fprintf(stderr, "Could not open %s for writing\n", path.c_str());
        return FLECS_IO;
    }

    auto fd = fileno(manifest);
    if (fd < 0)
    {
        std::fprintf(stderr, "Could not get fd for %s\n", path.c_str());
        return FLECS_IO;
    }

    const auto url = build_manifest_url(app_name, version);
    curl_easy_ext curl{url, &fd};
    if (!curl)
    {
        std::fprintf(stderr, "Could not initialize curl_easy_ext\n");
        return FLECS_CURL;
    }

    const auto curl_res = curl.perform();
    fclose(manifest);
    if (curl_res != CURLE_OK)
    {
        auto http_code = curl.response_code();
        std::fprintf(stderr, "Could not download app manifest: HTTP return code %ld\n", http_code);
        return static_cast<module_error_e>(FLECS_CURL + curl_res);
    }

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::xcheck_app_instance(
    const instances_table_entry_t& instance, const std::string& app_name, const std::string& version)
{
    // Is app installed?
    if (!app_name.empty() && !version.empty() && !is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Requested instance %s belongs to app %s (%s), which is not installed\n",
            instance.id.c_str(),
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    // Do app_name and instance's app match?
    if (!app_name.empty() && (instance.app != app_name))
    {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s belongs to app %s\n",
            instance.id.c_str(),
            app_name.c_str(),
            instance.app.c_str());
        return FLECS_INSTANCE_APP;
    }

    // Do version and instance's version match?
    if (!version.empty() && (instance.version != version))
    {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s (%s) belongs to version %s\n",
            instance.id.c_str(),
            instance.app.c_str(),
            version.c_str(),
            instance.version.c_str());
        return FLECS_INSTANCE_VERSION;
    }

    return FLECS_OK;
}

} // namespace Private
} // namespace FLECS
