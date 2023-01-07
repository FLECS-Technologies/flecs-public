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
#include "modules/jobs/jobs.h"
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
    if (manifest == nullptr) {
        std::fprintf(stderr, "Could not open %s for writing\n", path.c_str());
        return -1;
    }

    const auto url = build_manifest_url(app_name, version);
    auto response = cpr::Get(cpr::Url{url.c_str()});
    if (response.status_code != 200) {
        std::fprintf(stderr, "Could not download app manifest: HTTP return code %ld\n", response.status_code);
        return -1;
    }
    const auto bytes_written = fwrite(response.text.data(), 1, response.text.length(), manifest);
    fclose(manifest);
    if (bytes_written != response.text.length()) {
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
    for (decltype(auto) instance : _deployment->instances()) {
        if (_deployment->is_instance_running(instance.first)) {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_stop_instance(instance.first, "", "", _unused, true);
        }
    }

    do_save();
}

auto module_app_manager_private_t::do_init() //
    -> void
{
    // load apps and deployments
    do_load();

    // "install" system apps on first start
    constexpr auto system_apps = std::array<std::string_view, 2>{"tech.flecs.mqtt-bridge", "tech.flecs.service-mesh"};
    constexpr auto system_apps_desc = std::array<std::string_view, 2>{"FLECS MQTT Bridge", "FLECS Service Mesh"};

    for (size_t i = 0; i < system_apps.size(); ++i) {
        auto newer_version_installed = false;
        // delete old instances and uninstall old apps
        const auto versions = app_versions(system_apps[i]);
        for (const auto& version : versions) {
            if (version < FLECS_VERSION) {
                std::fprintf(stdout, "Removing old version %s system app %s\n", version.c_str(), system_apps[i].data());
                auto response = json_t{};
                for (const auto& instance_id : _deployment->instance_ids(app_key_t{system_apps[i].data(), version})) {
                    do_delete_instance(instance_id, {}, {}, response);
                }
                do_uninstall(system_apps[i].data(), version, response, true);
            }
            if (version > FLECS_VERSION) {
                newer_version_installed = true;
            }
        }

        // install current app and create an instance of it
        if (!newer_version_installed && !is_app_installed(system_apps[i].data(), FLECS_VERSION)) {
            std::fprintf(stdout, "Installing system app %s\n", system_apps[i].data());
            download_manifest(system_apps[i].data(), FLECS_VERSION);
            auto app = app_t{
                build_manifest_path(system_apps[i].data(), FLECS_VERSION),
                app_status_e::INSTALLED,
                app_status_e::INSTALLED};
            if (!app.app().empty()) {
                auto response = json_t{};
                _installed_apps.insert_or_assign(app_key_t{app.app(), app.version()}, std::move(app));
                do_create_instance(system_apps[i].data(), FLECS_VERSION, system_apps_desc[i].data(), response);
                const auto instance_id = _deployment->instance_ids(system_apps[i], FLECS_VERSION);
                if (!instance_id.empty()) {
                    auto& instance = _deployment->instances().at(instance_id[0]);
                    instance.desired(instance_status_e::RUNNING);
                }
            }
        }
    }
    do_save();

    std::fprintf(stdout, "Starting all app instances...\n");
    for (decltype(auto) instance : _deployment->instances()) {
        if (instance.second.desired() == instance_status_e::RUNNING) {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_start_instance(instance.first, instance.second.app_name(), instance.second.app_version(), _unused, true);
        }
    }

    auto hosts_thread = std::thread([] {
        pthread_setname_np(pthread_self(), "flecs-update-hosts");
        auto hosts_process = process_t{};
        hosts_process.spawnp("sh", "-c", "/opt/flecs/bin/flecs-update-hosts.sh");
        hosts_process.wait(false, false);
    });
    hosts_thread.detach();

    _mod_jobs = std::dynamic_pointer_cast<module_jobs_t>(api::query_module("jobs"));
}

auto module_app_manager_private_t::do_load(fs::path base_path) //
    -> void
{
    /// @todo remove for 2.0
    // query installed apps and instances from db before deleting it
    auto app_db = app_db_t{};
    if (app_db.is_open()) {
        for (decltype(auto) app : app_db.all_apps()) {
            const auto manifest_path = build_manifest_path(app.app, app.version);
            _installed_apps.emplace(
                std::piecewise_construct,
                std::forward_as_tuple(app.app, app.version),
                std::forward_as_tuple(manifest_path, app.status, app.desired));
        }
        persist_apps(base_path / "apps/");

        for (const auto& instance : app_db.all_instances()) {
            if (is_app_installed(instance.app, instance.version)) {
                const auto& app = _installed_apps.find(app_key_t{instance.app, instance.version})->second;
                auto tmp = instance_t{instance.id, &app, instance.description, instance.status, instance.desired};
                tmp.startup_options().push_back(instance.flags);
                for (std::size_t i = 0; i < instance.networks.size(); ++i) {
                    tmp.networks().push_back(instance_t::network_t{
                        .network_name = instance.networks[i],
                        .mac_address = {},
                        .ip_address = instance.ips[i]});
                }
                _deployment->insert_instance(tmp);
            }
        }
        _deployment->save(base_path / "deployment/");

        const auto db_path = app_db.path();
        const auto db_backup_path = db_path + ".migration";
        app_db.close();
        auto ec = std::error_code{};
        fs::rename(db_path, db_backup_path, ec);
    }

    load_apps(base_path / "apps/");
    _deployment->load(base_path / "deployment/");
    for (auto& instance : _deployment->instances()) {
        const auto it = _installed_apps.find(app_key_t{instance.second.app_name(), instance.second.app_version()});
        if (it != _installed_apps.cend()) {
            instance.second.app(&it->second);
        }
    }
}

auto module_app_manager_private_t::do_save(fs::path base_path) const //
    -> void
{
    persist_apps(base_path / "apps/");
    _deployment->save(base_path / "deployment/");
}

auto module_app_manager_private_t::is_app_installed(const std::string& app_name, const std::string& version) const //
    -> bool
{
    const auto it = _installed_apps.find(app_key_t{app_name, version});
    return (it != _installed_apps.cend()) && (it->second.status() == app_status_e::INSTALLED);
}

auto module_app_manager_private_t::app_versions(std::string_view app_name) const //
    -> std::vector<std::string>
{
    auto res = std::vector<std::string>{};
    std::for_each(_installed_apps.cbegin(), _installed_apps.cend(), [&](installed_apps_t::const_reference app) {
        if (app.first.name() == app_name) {
            res.push_back(app.second.version());
        }
    });
    return res;
}

auto module_app_manager_private_t::xcheck_app_instance(
    const instance_t& instance,
    const std::string& app_name,
    const std::string& version) //
    -> int
{
    // Is app installed?
    if (!app_name.empty() && !version.empty() && !is_app_installed(app_name, version)) {
        std::fprintf(
            stderr,
            "Requested instance %s belongs to app %s (%s), which is not installed\n",
            instance.id().c_str(),
            app_name.c_str(),
            version.c_str());
        return -1;
    }

    // Do app_name and instance's app match?
    if (!app_name.empty() && (instance.app_name() != app_name)) {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s belongs to app %s\n",
            instance.id().c_str(),
            app_name.c_str(),
            instance.app_name().c_str());
        return -1;
    }

    // Do version and instance's version match?
    if (!version.empty() && (instance.app_version() != version)) {
        std::fprintf(
            stderr,
            "Requested instance %s of app %s (%s) belongs to version %s\n",
            instance.id().c_str(),
            instance.app_version().c_str(),
            version.c_str(),
            instance.app_version().c_str());
        return -1;
    }

    return 0;
}

auto module_app_manager_private_t::persist_apps(fs::path base_path) const //
    -> void
{
    auto ec = std::error_code{};
    fs::create_directories(base_path, ec);
    if (ec) {
        return;
    }

    base_path /= "apps.json";
    auto apps_json = std::ofstream{base_path.c_str(), std::ios_base::out | std::ios_base::trunc};
    apps_json << json_t(_installed_apps);
}

auto module_app_manager_private_t::load_apps(fs::path base_path) //
    -> void
{
    base_path /= "apps.json";
    auto json_file = std::ifstream{base_path.c_str()};
    if (json_file.good()) {
        auto apps_json = parse_json(json_file);
        try {
            _installed_apps = apps_json.get<decltype(_installed_apps)>();
        } catch (const std::exception&) {
            _installed_apps.clear();
        }
    }
}

} // namespace Private
} // namespace FLECS
