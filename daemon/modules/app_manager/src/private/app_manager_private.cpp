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

#include "private/app_manager_private.h"

#include <arpa/inet.h>
#include <cpr/cpr.h>
#include <unistd.h>

#include <array>
#include <fstream>
#include <regex>
#include <set>
#include <sstream>
#include <thread>

#include "app/manifest/manifest.h"
#include "factory/factory.h"
#include "util/fs/fs.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "version/version.h"

namespace FLECS {
namespace Private {

auto build_manifest_path(const std::string& app_name, const std::string& version) //
    -> fs::path
{
    auto path = std::string{"/var/lib/flecs/apps"};

    path.append("/" + app_name);
    path.append("/" + version);

    auto ec = std::error_code{};
    fs::create_directories(path, ec);

    path.append("/manifest.yml");

    return path;
}

auto build_manifest_url(const std::string& app_name, const std::string& version) //
    -> std::string
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

auto download_manifest(const std::string& app_name, const std::string& version) //
    -> int
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
    if (response.status_code != 200)
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

module_app_manager_private_t::module_app_manager_private_t()
    : _deployment{new deployment_docker_t{}}
{}

module_app_manager_private_t::~module_app_manager_private_t()
{
    std::fprintf(stdout, "Stopping all running app instances...\n");
    for (decltype(auto) instance : _deployment->instances())
    {
        if (_deployment->is_instance_running(instance.first))
        {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_stop_instance(instance.first, "", "", _unused, true);
        }
    }

    persist_apps();
    _deployment->save();
}

auto module_app_manager_private_t::do_init() //
    -> void
{
    // query installed apps and instances from db before deleting it
    auto app_db = app_db_t{};
    if (app_db.is_open())
    {
        for (decltype(auto) app : app_db.all_apps())
        {
            const auto manifest_path = build_manifest_path(app.app, app.version);
            _installed_apps.emplace(
                std::piecewise_construct,
                std::forward_as_tuple(app.app, app.version),
                std::forward_as_tuple(app_t{manifest_path, app.status, app.desired}));
        }
        persist_apps();

        for (const auto& instance : app_db.all_instances())
        {
            auto tmp = instance_t{
                instance.id,
                instance.app,
                instance.version,
                instance.description,
                instance.status,
                instance.desired};
            tmp.startup_options().emplace_back(instance.flags);
            for (std::size_t i = 0; i < instance.networks.size(); ++i)
            {
                tmp.networks().emplace_back(instance_t::network_t{
                    .network_name = instance.networks[i],
                    .mac_address = {},
                    .ip_address = instance.ips[i]});
            }
            _deployment->insert_instance(tmp);
        }
        _deployment->save();

        const auto db_path = app_db.path();
        const auto db_backup_path = db_path + ".migration";
        app_db.close();
        auto ec = std::error_code{};
        fs::rename(db_path, db_backup_path, ec);
    }

    load_apps();
    _deployment->load();

    // "install" system apps on first start
    constexpr auto system_apps = std::array<const char*, 2>{"tech.flecs.mqtt-bridge", "tech.flecs.service-mesh"};
    constexpr auto system_apps_desc = std::array<const char*, 2>{"FLECS MQTT Bridge", "FLECS Service Mesh"};
    constexpr auto system_apps_versions = std::array<const char*, 2>{"1.0.0-porpoise", "1.0.0-porpoise"};

    for (size_t i = 0; i < system_apps.size(); ++i)
    {
        const auto instance_ids = _deployment->instance_ids(system_apps[i]);
        if (instance_ids.empty())
        {
            continue;
        }

        for (const auto& instance_id : instance_ids)
        {
            const auto& instance = _deployment->instances().at(instance_id);
            if (instance.version() != system_apps_versions[i])
            {
                std::fprintf(
                    stdout,
                    "Removing old version %s system app %s\n",
                    instance.version().c_str(),
                    instance.app().c_str());
                auto response = json_t{};
                do_uninstall(instance.app(), instance.version(), response, true);
            }
        }
    }

    for (size_t i = 0; i < system_apps.size(); ++i)
    {
        const auto instance_ids = _deployment->instance_ids(system_apps[i], system_apps_versions[i]);
        const auto instance_ready =
            instance_ids.empty()
                ? false
                : (_deployment->instances().at(instance_ids[0]).status() == instance_status_e::CREATED);

        if (!instance_ready)
        {
            std::fprintf(stdout, "Installing system app %s\n", system_apps[i]);
            download_manifest(system_apps[i], system_apps_versions[i]);
            const auto app = app_t{
                build_manifest_path(system_apps[i], system_apps_versions[i]),
                app_status_e::INSTALLED,
                app_status_e::INSTALLED};
            if (!app.app().empty())
            {
                auto response = json_t{};
                _installed_apps.emplace(
                    std::piecewise_construct,
                    std::forward_as_tuple(system_apps[i], system_apps_versions[i]),
                    std::forward_as_tuple(app));
                do_create_instance(app.app(), app.version(), system_apps_desc[i], response);
                const auto instance_id = _deployment->instance_ids(system_apps[i], system_apps_versions[i]);
                if (!instance_id.empty())
                {
                    auto& instance = _deployment->instances().at(instance_id[0]);
                    instance.desired(instance_status_e::RUNNING);
                }
            }
        }
    }
    persist_apps();

    std::fprintf(stdout, "Starting all app instances...\n");
    for (decltype(auto) instance : _deployment->instances())
    {
        if (instance.second.desired() == instance_status_e::RUNNING)
        {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_start_instance(instance.first, instance.second.app(), instance.second.version(), _unused, true);
        }
    }

    auto hosts_thread = std::thread([] {
        pthread_setname_np(pthread_self(), "flecs-update-hosts");
        auto hosts_process = process_t{};
        hosts_process.spawnp("sh", "-c", "/opt/flecs/bin/flecs-update-hosts.sh");
        hosts_process.wait(false, false);
    });
    hosts_thread.detach();
}

auto module_app_manager_private_t::is_app_installed(const std::string& app_name, const std::string& version) //
    -> bool
{
    return (_installed_apps.count(std::forward_as_tuple(app_name, version)) == 1) &&
           (_installed_apps[std::forward_as_tuple(app_name, version)].status() == app_status_e::INSTALLED);
}

auto module_app_manager_private_t::xcheck_app_instance(
    const instance_t& instance,
    const std::string& app_name,
    const std::string& version) //
    -> int
{
    // Is app installed?
    if (!app_name.empty() && !version.empty() && !is_app_installed(app_name, version))
    {
        std::fprintf(
            stderr,
            "Requested instance %s belongs to app %s (%s), which is not installed\n",
            instance.id().c_str(),
            app_name.c_str(),
            version.c_str());
        return -1;
    }

    // Do app_name and instance's app match?
    if (!app_name.empty() && (instance.app() != app_name))
    {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s belongs to app %s\n",
            instance.id().c_str(),
            app_name.c_str(),
            instance.app().c_str());
        return -1;
    }

    // Do version and instance's version match?
    if (!version.empty() && (instance.version() != version))
    {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s (%s) belongs to version %s\n",
            instance.id().c_str(),
            instance.app().c_str(),
            version.c_str(),
            instance.version().c_str());
        return -1;
    }

    return 0;
}

auto module_app_manager_private_t::persist_apps() const //
    -> void
{
    const auto path = "/var/lib/flecs/apps/";
    auto ec = std::error_code{};
    fs::create_directories(path, ec);
    if (ec)
    {
        return;
    }

    auto apps_json = std::ofstream{"/var/lib/flecs/apps/apps.json", std::ios_base::out | std::ios_base::trunc};
    apps_json << json_t(_installed_apps);
}

auto module_app_manager_private_t::load_apps() //
    -> void
{
    auto json_file = std::ifstream{"/var/lib/flecs/apps/apps.json"};
    if (json_file.good())
    {
        auto apps_json = parse_json(json_file);
        _installed_apps = apps_json.get<decltype(_installed_apps)>();
    }
}

} // namespace Private
} // namespace FLECS
