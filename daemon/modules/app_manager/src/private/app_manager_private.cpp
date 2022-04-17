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
#include <unistd.h>

#include <filesystem>
#include <regex>
#include <set>
#include <sstream>
#include <thread>

#include "app/app.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

module_app_manager_private_t::module_app_manager_private_t()
    : _app_db{}
{}

module_app_manager_private_t::~module_app_manager_private_t()
{
    std::fprintf(stdout, "Stopping all running app instances...\n");
    for (decltype(auto) instance : _app_db.all_instances())
    {
        if (is_instance_running(instance.id))
        {
            std::fprintf(stdout, "\t%s\n", instance.id.c_str());
            Json::Value _unused;
            do_stop_instance(instance.id, "", "", _unused, true);
        }
    }
}

void module_app_manager_private_t::do_init()
{
    std::fprintf(stdout, "Starting all app instances...\n");
    for (decltype(auto) instance : _app_db.all_instances())
    {
        if (instance.desired == instance_status_e::RUNNING)
        {
            std::fprintf(stdout, "\t%s\n", instance.id.c_str());
            Json::Value _unused;
            do_start_instance(instance.id, "", "", _unused, true);
        }
    }

    auto hosts_thread = std::thread([] {
        pthread_setname_np(pthread_self(), "flecs-update-hosts");
        auto hosts_process = process_t{};
        hosts_process.spawnp("sh", "-c", "flecs-update-hosts.sh");
        hosts_process.wait(false, false);
    });
    hosts_thread.detach();
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

int module_app_manager_private_t::xcheck_app_instance(
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
        return -1;
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
        return -1;
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
        return -1;
    }

    return 0;
}

std::string module_app_manager_private_t::generate_instance_ip()
{
    // Get base IP and subnet as "a.b.c.d/x"
    auto out = std::string{};
    {
        auto docker_process = FLECS::process_t{};
        docker_process
            .spawnp("docker", "network", "inspect", "-f", "'{{range .IPAM.Config}}{{.Subnet}}{{end}}'", "flecs");
        docker_process.wait(false, false);
        out = docker_process.stdout();
    }
    // parse a.b.c.d
    auto base_ip = in_addr_t{};
    {
        const auto ip_regex = std::regex{"(?:\\d{1,3}\\.){3}\\d{1,3}"};
        auto m = std::smatch{};
        if (!std::regex_search(out, m, ip_regex))
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
        if (!std::regex_search(out, m, subnet_regex) || m.size() < 2)
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
    for (decltype(auto) instance : _app_db.all_instances())
    {
        used_ips.emplace(inet_network(instance.ip.c_str()));
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

    return inet_ntoa(in_addr{.s_addr = htonl(instance_ip)});
}

} // namespace Private
} // namespace FLECS
