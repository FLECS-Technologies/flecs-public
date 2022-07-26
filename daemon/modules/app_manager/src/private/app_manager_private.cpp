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
#include <filesystem>
#include <fstream>
#include <regex>
#include <set>
#include <sstream>
#include <thread>

#include "app/manifest/manifest.h"
#include "factory/factory.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "version/version.h"

namespace FLECS {
namespace Private {

auto build_manifest_path(const std::string& app_name, const std::string& version) //
    -> std::filesystem::path
{
    auto path = std::string{"/var/lib/flecs/apps"};

    path.append("/" + app_name);
    path.append("/" + version);

    auto ec = std::error_code{};
    std::filesystem::create_directories(path, ec);

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
        if (is_instance_running(instance.first))
        {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_stop_instance(instance.first, "", "", _unused, true);
        }
    }

    persist_apps();
    persist_instances();
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
                tmp.networks().emplace_back(
                    instance_t::network_t{.network_name = instance.networks[i], .ip_address = instance.ips[i]});
            }
            _deployment->insert_instance(tmp);
        }
        persist_instances();

        const auto db_path = app_db.path();
        const auto db_backup_path = db_path + ".migration";
        app_db.close();
        auto ec = std::error_code{};
        std::filesystem::rename(db_path, db_backup_path, ec);
    }

    load_apps();
    load_instances();

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

    std::fprintf(stdout, "Starting all app instances...\n");
    for (decltype(auto) instance : _deployment->instances())
    {
        if (instance.second.desired() == instance_status_e::RUNNING)
        {
            std::fprintf(stdout, "\t%s\n", instance.first.c_str());
            json_t _unused;
            do_start_instance(instance.first, "", "", _unused, true);
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

auto module_app_manager_private_t::is_instance_runnable(const std::string& instance_id) //
    -> bool
{
    if (!_deployment->has_instance(instance_id))
    {
        return false;
    }

    const auto& instance = _deployment->instances().at(instance_id);
    if (instance.status() != instance_status_e::CREATED)
    {
        return false;
    }

    return true;
}

auto module_app_manager_private_t::is_instance_running(const std::string& instance_id) //
    -> bool
{
    auto docker_process = process_t{};

    docker_process.spawnp("docker", "ps", "--quiet", "--filter", std::string{"name=flecs-" + instance_id});
    docker_process.wait(false, false);
    // Consider instance running if Docker call was successful and returned a container id
    if (docker_process.exit_code() == 0 && !docker_process.stdout().empty())
    {
        return true;
    }

    return false;
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
    std::filesystem::create_directories(path, ec);
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

auto module_app_manager_private_t::persist_instances() const //
    -> void
{
    const auto path = "/var/lib/flecs/instances/";
    auto ec = std::error_code{};
    std::filesystem::create_directories(path, ec);
    if (ec)
    {
        return;
    }

    auto instances_json =
        std::ofstream{"/var/lib/flecs/instances/instances.json", std::ios_base::out | std::ios_base::trunc};
    instances_json << json_t(_deployment->instances());
}

auto module_app_manager_private_t::load_instances() //
    -> void
{
    auto json_file = std::ifstream{"/var/lib/flecs/instances/instances.json"};
    if (json_file.good())
    {
        auto instances_json = parse_json(json_file);
        _deployment->instances() = instances_json.get<std::remove_reference_t<decltype(_deployment->instances())>>();
    }
}

auto module_app_manager_private_t::generate_instance_ip() //
    -> std::string
{
    // Get base IP and subnet of FLECS network as "a.b.c.d/x"
    auto docker_process = FLECS::process_t{};
    docker_process.spawnp("docker", "network", "inspect", "-f", "'{{range .IPAM.Config}}{{.Subnet}}{{end}}'", "flecs");
    docker_process.wait(false, false);
    return generate_ip(docker_process.stdout());
}

auto module_app_manager_private_t::generate_ip(const std::string& cidr_subnet) //
    -> std::string
{
    // parse a.b.c.d
    auto base_ip = in_addr_t{};
    {
        const auto ip_regex = std::regex{"(?:\\d{1,3}\\.){3}\\d{1,3}"};
        auto m = std::smatch{};
        if (!std::regex_search(cidr_subnet, m, ip_regex))
        {
            return std::string{};
        }
        base_ip = inet_network(m[0].str().c_str());
    }
    // parse /x
    auto subnet_size = int{};
    {
        const auto subnet_regex = std::regex{"/(\\d{1,2})"};
        auto m = std::smatch{};
        if (!std::regex_search(cidr_subnet, m, subnet_regex) || m.size() < 2)
        {
            return std::string{};
        }
        subnet_size = std::stoi(m[1].str());
    }

    // determine the last ip address that belongs to the subnet:
    // (~0u << subnet_size) creates an inverted bitmask (such as 0xff00),
    // which is again inverted (yielding e.g. 0x00ff) and or'ed with the base ip.
    // finally subtract 1 to exclude the network's broadcast address
    const auto max_ip = (base_ip | ~(~0u << subnet_size)) - 1;

    auto used_ips = std::set<in_addr_t>{};
    for (decltype(auto) instance : _deployment->instances())
    {
        for (decltype(auto) network : instance.second.networks())
        {
            used_ips.emplace(ntohl(ipv4_to_bits(network.ip_address).s_addr));
        }
    }

    // skip network address and host address
    auto instance_ip = base_ip + 2;

    // search first unused address
    while (used_ips.find(instance_ip) != used_ips.end())
    {
        ++instance_ip;
    }

    if (instance_ip > max_ip)
    {
        return std::string{};
    }

    return ipv4_to_string(in_addr{.s_addr = htonl(instance_ip)});
}

} // namespace Private
} // namespace FLECS
