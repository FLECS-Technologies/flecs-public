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

#include "deployment_docker.h"

#include <filesystem>
#include <fstream>
#include <regex>

#include "app/app.h"
#include "factory/factory.h"
#include "system/system.h"
#include "util/cxx20/string.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "util/string/string_utils.h"

namespace FLECS {

auto deployment_docker_t::do_insert_instance(instance_t /*instance*/) //
    -> result_t
{
    return {0, ""};
}

auto deployment_docker_t::do_create_instance(const app_t& app, std::string instance_name) //
    -> result_t
{
    auto tmp =
        instance_t{app.app(), app.version(), instance_name, instance_status_e::REQUESTED, instance_status_e::CREATED};
    while (_instances.count(tmp.id()))
    {
        tmp.regenerate_id();
    }

    // @todo do this differently
    auto instance = _instances.emplace(tmp.id(), tmp).first;
    for (const auto& startup_option : app.startup_options())
    {
        instance->second.config().startup_options.emplace_back(
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option));
    }

    // Step 1: Create Docker volumes
    for (const auto& volume : app.volumes())
    {
        if (volume.type() != volume_t::VOLUME)
        {
            continue;
        }
        create_volume(instance->first, volume.host());
    }

    // Step 2: Create Docker networks
    // query and create flecs network, if not exists
    auto network = query_network("flecs");
    if (!network.has_value())
    {
        const auto [res, err] = create_network(network_type_t::BRIDGE, "flecs", "172.21.0.0/16", "172.21.0.1", "");
        if (res != 0)
        {
            return {-1, instance->first};
        }
        network = query_network("flecs");
        if (!network.has_value())
        {
            return {-1, instance->first};
        }
    }

    // query and create remaining networks
    for (const auto& network : app.networks())
    {
        auto network_exists = query_network(network.name()).has_value();
        if (!network_exists)
        {
            const auto [res, err] = create_network(network.type(), network.name(), "", "", network.parent());
            if (res != 0)
            {
                return {-1, instance->first};
            }
        }
    }

    // Step 3: Create conffiles
    const auto container_name = std::string{"flecs-"} + instance->first;
    const auto conf_path = std::string{"/var/lib/flecs/instances/"} + instance->first + std::string{"/conf/"};
    if (!app.conffiles().empty())
    {
        auto ec = std::error_code{};
        if (!std::filesystem::create_directories(conf_path, ec))
        {
            return {-1, instance->first};
        }
    }

    for (const auto& conffile : app.conffiles())
    {
        const auto local_path = conf_path + conffile.local();
        if (conffile.init())
        {
            const auto name = container_name + "-init";
            {
                auto docker_process = process_t{};
                docker_process.spawnp("docker", "create", "--name", name, app.image_with_tag());
                docker_process.wait(false, true);
                if (docker_process.exit_code() != 0)
                {
                    return {-1, instance->first};
                }
            }
            {
                auto docker_process = process_t{};
                docker_process.spawnp("docker", "cp", name + ":" + conffile.container(), local_path);
                docker_process.wait(false, true);
                if (docker_process.exit_code() != 0)
                {
                    return {-1, instance->first};
                }
            }
            {
                auto docker_process = process_t{};
                docker_process.spawnp("docker", "rm", "-f", name);
                docker_process.wait(false, true);
                if (docker_process.exit_code() != 0)
                {
                    return {-1, instance->first};
                }
            }
        }
        else
        {
            auto f = std::ofstream{local_path};
            if (!f.good())
            {
                return {-1, instance->first};
            }
        }
    }

    instance->second.status(instance_status_e::RESOURCES_READY);

    // Step 4: Create Docker container
    auto docker_process = process_t{};
    docker_process.arg("create");

    for (const auto& env : app.env())
    {
        docker_process.arg("--env");
        docker_process.arg(stringify(env));
    }
    for (const auto& conffile : app.conffiles())
    {
        docker_process.arg("--volume");
        const auto arg = conf_path + conffile.local() + ":" + conffile.container() + (conffile.ro() ? ":ro" : "");
        docker_process.arg(arg);
    }
    for (const auto& volume : app.volumes())
    {
        docker_process.arg("--volume");
        docker_process.arg(container_name + "-" + volume.host() + ":" + volume.container());
    }
    for (const auto& port_range : app.ports())
    {
        docker_process.arg("--publish");
        docker_process.arg(stringify(port_range));
    }
    if (app.interactive())
    {
        docker_process.arg("--interactive");
    }

    docker_process.arg("--name");
    docker_process.arg(container_name);

    if (!app.hostname().empty())
    {
        docker_process.arg("--hostname");
        docker_process.arg(app.hostname());
    }
    else
    {
        docker_process.arg("--hostname");
        docker_process.arg(container_name);
    }

    for (const auto& device : app.devices())
    {
        docker_process.arg("--device");
        docker_process.arg(device);
    }

    if (!app.networks().empty())
    {
        const auto& network = app.networks()[0];

        const auto net = query_network(network.name());
        const auto ip = generate_instance_ip(net->cidr_subnet, net->gateway);
        if (ip.empty())
        {
            return {-1, instance->first};
        }

        docker_process.arg("--network");
        docker_process.arg(app.networks()[0].name());

        docker_process.arg("--ip");
        docker_process.arg(ip);

        if (!network.mac_address().empty())
        {
            docker_process.arg("--mac-address");

            if (cxx20::starts_with(network.mac_address(), "clone:"))
            {
                const auto parts = split(network.mac_address(), ':');
                if (parts.size() != 2)
                {
                    return {-1, instance->first};
                }

                const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
                const auto adapters = system_api->get_network_adapters();
                const auto netif = adapters.find(parts[1]);
                if (netif == adapters.cend())
                {
                    return {-1, instance->first};
                }
                docker_process.arg(netif->second.mac);
            }
            else
            {
                docker_process.arg(network.mac_address());
            }
        }

        instance->second.config().networks.emplace_back(instance_config_t::network_t{.network = net->name, .ip = ip});
    }

    if (std::count(
            app.startup_options().cbegin(),
            app.startup_options().cend(),
            startup_option_t::INIT_NETWORK_AFTER_START))
    {
        instance->second.config().startup_options.emplace_back(
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option_t::INIT_NETWORK_AFTER_START));
        docker_process.arg("--mount");
        docker_process.arg("type=tmpfs,destination=/flecs-tmp");

        auto cmd = std::string{};
        {
            auto docker_process = process_t{};

            docker_process.arg("inspect");
            docker_process.arg("--format");
            docker_process.arg("{{.Config.Cmd}}");
            docker_process.arg(app.image_with_tag());

            docker_process.spawnp("docker");
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0)
            {
                return {-1, instance->first};
            }

            cmd = docker_process.stdout();
        }
        trim(cmd);
        cmd.erase(cmd.find_first_of('['), 1);
        cmd.erase(cmd.find_last_of(']'), 1);
        if (cxx20::starts_with(cmd, "/bin/sh -c "))
        {
            cmd.erase(0, 11);
        }

        const auto entrypoint_path =
            std::string{"/var/lib/flecs/instances/"} + instance->first + std::string{"/scripts/"};

        auto ec = std::error_code{};
        if (!std::filesystem::create_directories(entrypoint_path, ec))
        {
            return {-1, instance->first};
        }

        auto entrypoint = std::ofstream{entrypoint_path + "entrypoint.sh"};
        entrypoint << "#!/bin/sh\n\n";
        entrypoint << "while [ ! -f /flecs-tmp/ready ]; do\n\n";
        entrypoint << "    sleep 1;\n";
        entrypoint << "done\n\n";
        entrypoint << cmd << std::endl;

        docker_process.arg("--entrypoint");
        docker_process.arg("/flecs-entrypoint.sh");
    }

    docker_process.arg(app.image_with_tag());

    for (const auto& arg : app.args())
    {
        docker_process.arg(arg);
    }

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, instance->first};
    }

    if (std::count(
            app.startup_options().cbegin(),
            app.startup_options().cend(),
            startup_option_t::INIT_NETWORK_AFTER_START))
    {
        const auto entrypoint_path =
            std::string{"/var/lib/flecs/instances/"} + instance->first + std::string{"/scripts/entrypoint.sh"};

        auto ec = std::error_code{};

        namespace fs = std::filesystem;
        fs::permissions(entrypoint_path, fs::perms::owner_exec | fs::perms::group_exec | fs::perms::others_exec, ec);

        if (ec)
        {
            return {-1, instance->first};
        }

        {
            auto docker_process = process_t{};

            docker_process.arg("cp");
            docker_process.arg(entrypoint_path);
            docker_process.arg(container_name + ":/flecs-entrypoint.sh");

            docker_process.spawnp("docker");
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0)
            {
                return {-1, instance->first};
            }
        }
    }

    // assign static ips to remaining networks
    for (size_t i = 1; i < app.networks().size(); ++i)
    {
        const auto& network = app.networks()[i];
        const auto net = query_network(network.name());
        const auto ip = generate_instance_ip(net->cidr_subnet, net->gateway);
        if (ip.empty())
        {
            return {-1, instance->first};
        }
        if (!std::count(
                app.startup_options().cbegin(),
                app.startup_options().cend(),
                startup_option_t::INIT_NETWORK_AFTER_START))
        {
            connect_network(instance->first, net->name, ip);
        }
        instance->second.config().networks.emplace_back(instance_config_t::network_t{.network = net->name, .ip = ip});
        if (network.type() == network_type_t::IPVLAN || network.type() == network_type_t::MACVLAN)
        {
            instance->second.config().networkAdapters.emplace_back(instance_config_t::network_adapter_t{
                .name = network.parent(),
                .ipAddress = ip,
                .subnetMask = cidr_to_subnet_mask_v4(net->cidr_subnet),
                .gateway = net->gateway,
                .active = true});
        }
    }

    instance->second.status(instance_status_e::CREATED);

    return {0, instance->first};
}

auto deployment_docker_t::do_delete_instance(std::string_view instance_id) //
    -> result_t
{
    _instances.erase(instance_id.data());
    return {0, ""};
}

auto deployment_docker_t::do_start_instance(std::string_view instance_id) //
    -> result_t
{
    const auto container_name = std::string{"flecs-"} + std::string{instance_id};

    auto docker_process = process_t{};

    docker_process.arg("start");
    docker_process.arg(container_name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_ready_instance(std::string_view instance_id) //
    -> result_t
{
    const auto container_name = std::string{"flecs-"} + std::string{instance_id};

    auto docker_process = process_t{};

    docker_process.arg("exec");
    docker_process.arg(container_name);
    docker_process.arg("touch");
    docker_process.arg("/flecs-tmp/ready");

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {0, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_stop_instance(std::string_view instance_id) //
    -> result_t
{
    auto res = int{};
    auto errmsg = std::string{};

    const auto container_name = std::string{"flecs-"} + std::string{instance_id};

    auto docker_process = process_t{};

    docker_process.arg("stop");
    docker_process.arg(container_name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {res, errmsg};
}

auto deployment_docker_t::do_create_network(
    network_type_t network_type,
    std::string_view network,
    std::string_view cidr_subnet,
    std::string_view gateway,
    std::string_view parent_adapter) //
    -> result_t
{
    // @todo review and cleanup
    auto docker_process = process_t{};

    auto subnet = std::string{cidr_subnet};
    auto gw = std::string{gateway};

    switch (network_type)
    {
        case network_type_t::BRIDGE: {
            break;
        }
        case network_type_t::IPVLAN: {
            if (parent_adapter.empty())
            {
                return {-1, "cannot create ipvlan network without parent"};
            }
            if (cidr_subnet.empty() || gateway.empty())
            {
                const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
                const auto adapters = system_api->get_network_adapters();
                const auto netif = adapters.find(parent_adapter.data());
                if (netif == adapters.cend())
                {
                    return {-1, "network adapter does not exist"};
                }
                if (netif->second.ipv4_addr.empty())
                {
                    return {-1, "network adapter is not ready"};
                }

                // create ipvlan network, if not exists
                subnet = ipv4_to_network(netif->second.ipv4_addr[0].addr, netif->second.ipv4_addr[0].subnet_mask);
                gw = netif->second.gateway;
            }
            break;
        }
        case network_type_t::MACVLAN: {
            break;
        }
        case network_type_t::INTERNAL: {
            break;
        }
        default: {
            break;
        }
    }
    docker_process.arg("network");
    docker_process.arg("create");
    docker_process.arg("--driver");
    docker_process.arg(stringify(network_type));
    docker_process.arg("--subnet");
    docker_process.arg(subnet);
    docker_process.arg("--gateway");
    docker_process.arg(gw);
    if (!parent_adapter.empty())
    {
        docker_process.arg("--opt");
        docker_process.arg("parent=" + std::string{parent_adapter});
    }
    docker_process.arg(std::string{network});

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_query_network(std::string_view network) //
    -> std::optional<network_t>
{
    auto res = network_t{};
    res.name = network;
    {
        // Get type of network
        auto docker_process = process_t{};

        docker_process.arg("network");
        docker_process.arg("inspect");
        docker_process.arg("--format");
        docker_process.arg("{{.IPAM.Driver}}");
        docker_process.arg(std::string{network});

        docker_process.spawnp("docker");
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0)
        {
            return {};
        }
        auto out = docker_process.stdout();
        res.type = network_type_from_string(trim(out));
    }
    {
        // Get base IP and subnet of network as "a.b.c.d/x"
        auto docker_process = process_t{};

        docker_process.arg("network");
        docker_process.arg("inspect");
        docker_process.arg("--format");
        docker_process.arg("{{range .IPAM.Config}}{{.Subnet}}{{end}}");
        docker_process.arg(std::string{network});

        docker_process.spawnp("docker");
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0)
        {
            return {};
        }
        auto out = docker_process.stdout();
        res.cidr_subnet = trim(out);
    }
    {
        // Get gateway of network as "a.b.c.d"

        auto docker_process = process_t{};

        docker_process.arg("network");
        docker_process.arg("inspect");
        docker_process.arg("--format");
        docker_process.arg("{{range .IPAM.Config}}{{.Gateway}}{{end}}");
        docker_process.arg(std::string{network});

        docker_process.spawnp("docker");
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0)
        {
            return {};
        }
        auto out = docker_process.stdout();
        res.gateway = trim(out);
    }

    return res;
}

auto deployment_docker_t::do_delete_network(std::string_view /*network*/) //
    -> result_t
{
    return {0, ""};
}

auto deployment_docker_t::do_connect_network(
    std::string_view instance_id,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    auto docker_process = process_t{};

    docker_process.arg("network");
    docker_process.arg("connect");
    docker_process.arg("--ip");
    docker_process.arg(std::string{ip});
    docker_process.arg(std::string{network});
    docker_process.arg("flecs-" + std::string{instance_id});

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_disconnect_network(std::string_view instance_id, std::string_view network) //
    -> result_t
{
    auto docker_process = process_t{};

    docker_process.arg("network");
    docker_process.arg("disconnect");
    docker_process.arg("--force");
    docker_process.arg(std::string{network});
    docker_process.arg("flecs-" + std::string{instance_id});

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_create_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    const auto name = std::string{"flecs-"} + std::string{instance_id} + "-" + std::string{volume_name};

    auto docker_process = process_t{};

    docker_process.arg("volume");
    docker_process.arg("create");
    docker_process.arg(name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_delete_volume(std::string_view /*instance_id*/, std::string_view /*volume_name*/) //
    -> result_t
{
    return {0, ""};
}

} // namespace FLECS
