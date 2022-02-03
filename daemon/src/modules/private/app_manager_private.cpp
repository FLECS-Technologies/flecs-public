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

module_error_e module_app_manager_private_t::do_install(const std::string& app_name, const std::string& version)
{
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

    auto app = app_t{manifest};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    auto status = MANIFEST_DOWNLOADED;
    _app_db.insert_app({app.name(), app.version(), status, desired, app.category(), 0});

    auto docker_process = process_t{};
    docker_process.spawnp("docker", "pull", app.image_with_tag());
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return FLECS_DOCKER;
    }

    // status = IMAGE_DOWNLOADED;

    status = INSTALLED;

    _app_db.insert_app(apps_table_entry_t{app.name(), app.version(), status, desired, app.category(), 0});
    _app_db.persist();

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_sideload(const std::string& manifest_path)
{
    auto app = app_t{manifest_path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    const auto status = app_status_e::NOT_INSTALLED;
    const auto desired = app_status_e::INSTALLED;
    _app_db.insert_app({app.name(), app.version(), status, desired, app.category(), 0});

    const auto path = build_manifest_path(app.name(), app.version());

    std::error_code ec;
    std::filesystem::remove(path, ec);
    std::filesystem::copy(manifest_path, path, ec);
    if (ec)
    {
        std::fprintf(stderr, "Could not copy manifest to %s: %d\n", path.c_str(), ec.value());
        return FLECS_IO;
    }

    return do_install(manifest_path);
}

module_error_e module_app_manager_private_t::do_uninstall(const std::string& app_name, const std::string& version)
{
    if (!is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Could not uninstall %s (%s), which is not installed\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    const auto path = build_manifest_path(app_name, version);

    auto app = app_t{path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    const auto instances = _app_db.instances(app_name, version);
    for (auto& instance : instances)
    {
        const auto res = do_stop_instance(app_name, version, instance.id);
        if (res != FLECS_OK)
        {
            std::fprintf(stderr, "Warning: Could not stop instance %s: %d\n", instance.id.c_str(), res);
        }
        _app_db.delete_instance({instance.id});
    }

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

    const auto res = unlink(path.c_str());
    if (res < 0)
    {
        std::fprintf(stderr, "Could not delete manifest %s: %d (%s)\n", path.c_str(), errno, strerror(errno));
        return FLECS_IO;
    }

    _app_db.delete_app({app_name, version});
    _app_db.persist();

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_create_instance(
    const std::string& app_name, const std::string& version, const std::string& description)
{
    auto status = instance_status_e::NOT_CREATED;
    const auto desired = instance_status_e::CREATED;

    if (!is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Could not create instance of app %s (%s), which is not installed\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    const auto path = build_manifest_path(app_name, version);
    app_t app{path};
    if (!app.yaml_loaded())
    {
        std::fprintf(
            stderr,
            "Could not create instance of app %s (%s): manifest error\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_YAML;
    }

    auto seed = std::random_device{};
    auto generator = std::mt19937{seed()};
    auto distribution = std::uniform_int_distribution{
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()};

    const auto id = distribution(generator);
    auto ss = std::stringstream{};
    ss << std::hex << std::setw(8) << std::setfill('0') << id;

    status = instance_status_e::REQUESTED;
    _app_db.insert_instance({ss.str(), app.name(), app.version(), description, status, desired, 0});

    for (const auto& volume : app.volumes())
    {
        auto docker_process = process_t{};
        const auto name = std::string{"flecs-"} + ss.str() + "-" + volume.first;
        docker_process.spawnp("docker", "volume", "create", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            std::fprintf(stderr, "Could not create docker volume %s\n", name.c_str());
            return FLECS_DOCKER;
        }
    }

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
    _app_db.insert_instance({ss.str(), app.name(), app.version(), description, status, desired, 0});

    auto docker_process = process_t{};
    docker_process.arg("create");
    for (const auto& volume : app.volumes())
    {
        docker_process.arg("--volume");
        docker_process.arg("flecs-" + ss.str() + "-" + volume.first + ":" + volume.second);
    }
    for (const auto& bind_mount : app.bind_mounts())
    {
        docker_process.arg("--volume");
        docker_process.arg(bind_mount.first + ":" + bind_mount.second + " ");
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
    docker_process.arg("--name");
    docker_process.arg("flecs-" + ss.str());
    docker_process.arg(app.image_with_tag());

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        std::fprintf(stderr, "Could not create container for instance %s\n", ss.str().c_str());
        return FLECS_DOCKER;
    }

    // status = instance_status_e::CREATED;
    status = instance_status_e::STOPPED;
    _app_db.insert_instance({ss.str(), app.name(), app.version(), description, status, desired, 0});

    std::cout << ss.str();

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_delete_instance(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    if (!_app_db.has_instance({id}))
    {
        return FLECS_INSTANCE_NOTEXIST;
    }
    auto instance = _app_db.query_instance({id}).value();
    {
        if (is_instance_running(app_name, version, id))
        {
            const auto res = do_stop_instance(app_name, version, id);
            if (res != FLECS_OK)
            {
                std::fprintf(stderr, "Could not stop instance %s: %d\n", id.c_str(), res);
                return res;
            }
        }
    }

    const auto path = build_manifest_path(instance.app, instance.version);
    app_t app{path};
    if (!app.yaml_loaded())
    {
        std::fprintf(
            stderr,
            "Could not delete instance of app %s (%s): manifest error\n",
            app_name.c_str(),
            version.c_str());
        return FLECS_YAML;
    }

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

    _app_db.delete_instance({id});

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_start_instance(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    if (!_app_db.has_instance({id}))
    {
        return FLECS_INSTANCE_NOTEXIST;
    }

    auto instance = _app_db.query_instance({id}).value();
    if (!app_name.empty() && !version.empty() && !is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Request to start instance %s of app %s (%s), which is not installed\n",
            id.c_str(),
            app_name.c_str(),
            version.c_str());
        return FLECS_APP_NOTINST;
    }

    if (!app_name.empty() && (instance.app != app_name))
    {
        std::fprintf(
            stderr,
            "Request to start instance %s of app %s, which belongs to %s\n",
            id.c_str(),
            app_name.c_str(),
            instance.app.c_str());
        return FLECS_INSTANCE_APP;
    }

    if (!version.empty() && (instance.version != version))
    {
        std::fprintf(
            stderr,
            "Request to start instance %s of app %s (%s), which belongs to version %s\n",
            id.c_str(),
            instance.app.c_str(),
            version.c_str(),
            instance.version.c_str());
        return FLECS_INSTANCE_VERSION;
    }

    instance.desired = instance_status_e::RUNNING;
    _app_db.insert_instance(instance);

    const auto path = build_manifest_path(instance.app, instance.version);
    app_t app{path};
    if (!app.yaml_loaded())
    {
        return FLECS_YAML;
    }

    auto docker_process = process_t{};
    const auto name = std::string{"flecs"} + "-" + id;

    docker_process.spawnp("docker", "start", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return FLECS_DOCKER;
    }

    instance.status = instance_status_e::RUNNING;
    _app_db.insert_instance(instance);

    return FLECS_OK;
}

#define XCHECK_APP_INSTALLED(app_entry, app_name, version)                                                        \
    if (app_entry.app != app_name || app_entry.version != version || app_entry.status != app_status_e::INSTALLED) \
    {                                                                                                             \
        std::fprintf(stderr, "App %s (%s) is not installed\n", app_name.c_str(), version.c_str());                \
        return FLECS_APP_NOTINST;                                                                                 \
    }

#define XCHECK_INSTANCE_EXISTS(instance_entry, id) \
    do                                             \
    {                                              \
        if (instance_entry.id != id)               \
        {                                          \
            return FLECS_INSTANCE_NOTEXIST;        \
        };                                         \
    } while (false)

#define XCHECK_INSTANCE_RUNNING(instance_entry)                  \
    do                                                           \
    {                                                            \
        if (instance_entry.status != instance_status_e::RUNNING) \
        {                                                        \
            return FLECS_INSTANCE_NOTRUN;                        \
        }                                                        \
    } while (false)

module_error_e module_app_manager_private_t::do_stop_instance(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    if (!_app_db.has_instance({id}))
    {
        return FLECS_INSTANCE_NOTEXIST;
    }
    auto instance = _app_db.query_instance({id}).value();

    XCHECK_INSTANCE_EXISTS(instance, id);
    XCHECK_INSTANCE_RUNNING(instance);

    instance.desired = instance_status_e::STOPPED;
    _app_db.insert_instance(instance);

    auto docker_process = process_t{};
    const auto name = std::string{"flecs"} + "-" + id;
    docker_process.spawnp("docker", "stop", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return FLECS_DOCKER;
    }

    instance.status = instance_status_e::STOPPED;
    _app_db.insert_instance(instance);

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_list_apps(const std::string& app_name)
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
            json_instance["status"] = instance_status_to_string(instance.status);
            json_instance["desired"] = instance_status_to_string(instance.desired);
            json_instance["version"] = instance.version;
            json_app["instances"].append(json_instance);
        }
        json_value["appList"].append(json_app);
    }
    fprintf(stdout, "%s", json_value.toStyledString().c_str());

    return FLECS_OK;
}

module_error_e module_app_manager_private_t::do_list_instances(
    const std::string& /*app_name*/, const std::string& /*version*/)
{
    return FLECS_OK;
}

bool module_app_manager_private_t::is_app_installed(const std::string& app_name, const std::string& version)
{
    return _app_db.has_app({app_name, version});
}

bool module_app_manager_private_t::is_instance_available(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    if (_app_db.has_instance({id}))
    {
        const auto instance = _app_db.query_instance({id});
        return instance->app == app_name && instance->version == version;
    }
    return false;
}

bool module_app_manager_private_t::is_instance_runnable(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    const auto instance_entry = _app_db.query_instance({id}).value();
    if ((instance_entry.app != app_name) || (instance_entry.version != version) || (instance_entry.id != id) ||
        (instance_entry.status != instance_status_e::CREATED && instance_entry.status != instance_status_e::STOPPED))
    {
        return false;
    }

    return true;
}

bool module_app_manager_private_t::is_instance_running(
    const std::string& app_name, const std::string& version, const std::string& id)
{
    const auto instance_entry = _app_db.query_instance({id}).value();
    if ((instance_entry.app != app_name) || (instance_entry.version != version) || (instance_entry.id != id) ||
        (instance_entry.status != instance_status_e::RUNNING))
    {
        return false;
    }

    return true;
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

    path.append("/");
    path.append("manifest.yml");

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

} // namespace Private
} // namespace FLECS
