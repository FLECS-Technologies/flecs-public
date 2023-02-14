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

#include "app/app.h"
#include "app/manifest/manifest.h"
#include "factory/factory.h"
#include "system/system.h"
#include "util/cxx20/string.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "util/string/string_utils.h"
#include "util/sysfs/sysfs.h"

namespace FLECS {

auto deployment_docker_t::create_container(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto container_name = std::string{"flecs-"} + instance->id().hex();

    // cleanup after possible unclean shutdown
    if (!is_instance_running(instance)) {
        delete_container(instance);
    }

    {
        auto docker_process = process_t{};
        docker_process.arg("ps");
        docker_process.arg("--all");
        docker_process.arg("--format");
        docker_process.arg("{{.Names}}");
        docker_process.spawnp("docker");
        docker_process.wait(false, false);
        if (docker_process.exit_code() == 0) {
            if (cxx20::contains(docker_process.stdout(), container_name.c_str())) {
                return {0, "Container already exists"};
            };
        }
    }

    auto docker_process = process_t{};
    docker_process.arg("create");

    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (const auto& env : manifest->env()) {
        docker_process.arg("--env");
        docker_process.arg(stringify(env));
    }
    for (const auto& volume : manifest->volumes()) {
        docker_process.arg("--volume");
        docker_process.arg(container_name + "-" + volume.host() + ":" + volume.container());
    }
    for (const auto& port_range : manifest->ports()) {
        docker_process.arg("--publish");
        docker_process.arg(stringify(port_range));
    }
    if (manifest->interactive()) {
        docker_process.arg("--interactive");
    }

    docker_process.arg("--name");
    docker_process.arg(container_name);

    if (!manifest->hostname().empty()) {
        docker_process.arg("--hostname");
        docker_process.arg(manifest->hostname());
    } else {
        docker_process.arg("--hostname");
        docker_process.arg(container_name);
    }

    for (const auto& device : manifest->devices()) {
        docker_process.arg("--device");
        docker_process.arg(device);
    }

    if (!instance->networks().empty()) {
        auto& network = instance->networks()[0];

        if (network.ip_address.empty()) {
            const auto net = query_network(network.network_name);
            network.ip_address = generate_instance_ip(net->cidr_subnet, net->gateway);
            if (network.ip_address.empty()) {
                return {-1, "Could not generate instance IP"};
            }
        }

        docker_process.arg("--network");
        docker_process.arg(instance->networks()[0].network_name);

        docker_process.arg("--ip");
        docker_process.arg(network.ip_address);

        if (!network.mac_address.empty()) {
            docker_process.arg("--mac-address");

            if (cxx20::starts_with(network.mac_address, "clone:")) {
                const auto parts = split(network.mac_address, ':');
                if (parts.size() != 2) {
                    return {-1, "Cloned MAC address is invalid"};
                }

                const auto system_api =
                    dynamic_cast<const module_system_t*>(api::query_module("system").get());
                const auto adapters = system_api->get_network_adapters();
                const auto netif = adapters.find(parts[1]);
                if (netif == adapters.cend()) {
                    return {-1, "Could not find network adapter for cloned MAC address"};
                }
                network.mac_address = netif->second.mac;
            } else {
                network.mac_address = network.mac_address;
            }
            docker_process.arg(network.mac_address);
        }
    }

    for (const auto& usb_device : instance->usb_devices()) {
        const auto busnum = sysfs::usb_busnum(usb_device.port);
        const auto devnum = sysfs::usb_devnum(usb_device.port);
        if (busnum.has_value() && devnum.has_value()) {
            auto path = std::string{"/dev/bus/usb/***/***"};
            std::snprintf(
                path.data(),
                path.size() + 1,
                "/dev/bus/usb/%03d/%03d",
                sysfs::usb_busnum(usb_device.port).value(),
                sysfs::usb_devnum(usb_device.port).value());

            auto ec = std::error_code{};
            if (std::filesystem::exists(path, ec)) {
                docker_process.arg("--device");
                docker_process.arg(path);
            }
        }
    }

    if (std::find(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            (unsigned int)startup_option_t::INIT_NETWORK_AFTER_START) !=
        instance->startup_options().cend()) {
        docker_process.arg("--mount");
        docker_process.arg("type=tmpfs,destination=/flecs-tmp");

        auto cmd = std::string{};
        {
            auto docker_process = process_t{};

            docker_process.arg("inspect");
            docker_process.arg("--format");
            docker_process.arg("{{.Config.Cmd}}");
            docker_process.arg(manifest->image_with_tag());

            docker_process.spawnp("docker");
            docker_process.wait(false, true);
            if (docker_process.exit_code() != 0) {
                return {-1, "Could not determine entrypoint"};
            }

            cmd = docker_process.stdout();
        }
        trim(cmd);
        cmd.erase(cmd.find_first_of('['), 1);
        cmd.erase(cmd.find_last_of(']'), 1);
        if (cxx20::starts_with(cmd, "/bin/sh -c ")) {
            cmd.erase(0, 11);
        }

        const auto entrypoint_path = std::string{"/var/lib/flecs/instances/"} +
                                     instance->id().hex() + std::string{"/scripts/"};

        auto ec = std::error_code{};
        fs::create_directories(entrypoint_path, ec);
        if (ec) {
            return {-1, "Could not create entrypoint directory"};
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

    docker_process.arg(manifest->image_with_tag());

    for (const auto& arg : manifest->args()) {
        docker_process.arg(arg);
    }

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, "Could not create Docker container"};
    }

    const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
    for (const auto& conffile : manifest->conffiles()) {
        const auto [res, err] =
            copy_file_to_instance(instance, conf_path + conffile.local(), conffile.container());
        if (res != 0) {
            std::fprintf(
                stderr,
                "Warning: Could not copy file %s to %s of instance %s: %s\n",
                conffile.local().c_str(),
                conffile.container().c_str(),
                instance->id().hex().c_str(),
                err.c_str());
        }
    }

    if (std::find(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            std::underlying_type_t<startup_option_t>(startup_option_t::INIT_NETWORK_AFTER_START)) !=
        instance->startup_options().cend()) {
        const auto entrypoint_path = std::string{"/var/lib/flecs/instances/"} +
                                     instance->id().hex() + std::string{"/scripts/entrypoint.sh"};

        auto ec = std::error_code{};

        fs::permissions(
            entrypoint_path,
            fs::perms::owner_exec | fs::perms::group_exec | fs::perms::others_exec,
            ec);

        if (ec) {
            return {-1, "Could not make entrypoint executable"};
        }

        const auto [res, err_msg] =
            copy_file_to_instance(instance, entrypoint_path, "/flecs-entrypoint.sh");
        if (res != 0) {
            return {-1, "Could not copy entrypoint to container"};
        }
    }

    // assign static ips to remaining networks
    for (size_t i = 1; i < instance->networks().size(); ++i) {
        auto& network = instance->networks()[i];
        const auto net = query_network(network.network_name);
        if (!net.has_value()) {
            return {-1, "Requested network does not exist"};
        }
        if (network.ip_address.empty()) {
            network.ip_address = generate_instance_ip(net->cidr_subnet, net->gateway);
            if (network.ip_address.empty()) {
                return {-1, "Could not generate IP for additional networks"};
            }
        }
        if (std::find(
                instance->startup_options().cbegin(),
                instance->startup_options().cend(),
                std::underlying_type_t<startup_option_t>(
                    startup_option_t::INIT_NETWORK_AFTER_START)) ==
            instance->startup_options().cend()) {
            connect_network(instance, net->name, network.ip_address);
        }
    }

    return {0, {}};
}

auto deployment_docker_t::delete_container(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (app) {
        auto manifest = app->manifest();
        if (manifest) {
            const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
            for (const auto& conffile : manifest->conffiles()) {
                copy_file_from_instance(
                    instance,
                    conffile.container(),
                    conf_path + conffile.local());
            }
        }
    }

    const auto container_name = "flecs-" + instance->id().hex();

    auto docker_process = process_t{};
    docker_process.arg("rm");
    docker_process.arg("--force");
    docker_process.arg(container_name);
    docker_process.spawnp("docker");
    docker_process.wait(false, false);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_deployment_id() const noexcept //
    -> std::string_view
{
    return "docker";
}

auto deployment_docker_t::do_create_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    instance->status(instance_status_e::Created);
    return {0, instance->id().hex()};
}

auto deployment_docker_t::do_delete_instance(std::shared_ptr<instance_t> /*instance*/) //
    -> result_t
{
    return {0, ""};
}

auto deployment_docker_t::do_start_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto [res, additional_info] = create_container(instance);
    if (res != 0) {
        return {res, additional_info};
    }

    const auto container_name = "flecs-" + instance->id().hex();

    auto docker_process = process_t{};

    docker_process.arg("start");
    docker_process.arg(container_name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_ready_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto container_name = "flecs-" + instance->id().hex();

    auto docker_process = process_t{};

    docker_process.arg("exec");
    docker_process.arg(container_name);
    docker_process.arg("touch");
    docker_process.arg("/flecs-tmp/ready");

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {0, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_stop_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto container_name = "flecs-" + instance->id().hex();

    auto docker_process = process_t{};

    docker_process.arg("stop");
    docker_process.arg(container_name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);

    return delete_container(instance);
}

auto deployment_docker_t::do_export_instance(
    std::shared_ptr<instance_t> /*instance*/, fs::path /*dest_dir*/) const //
    -> result_t
{
    return {0, {}};
}

auto deployment_docker_t::do_is_instance_running(std::shared_ptr<instance_t> instance) const //
    -> bool
{
    auto docker_process = process_t{};
    docker_process.arg("ps");
    docker_process.arg("--quiet");
    docker_process.arg("--filter");
    docker_process.arg("name=flecs-" + instance->id().hex());
    docker_process.spawnp("docker");
    docker_process.wait(false, false);
    // Consider instance running if Docker call was successful and returned a container id
    if (docker_process.exit_code() == 0 && !docker_process.stdout().empty()) {
        return true;
    }

    return false;
}

auto deployment_docker_t::do_create_network(
    network_type_e network_type,
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

    switch (network_type) {
        case network_type_e::Bridge: {
            break;
        }
        case network_type_e::IPVLAN: {
            if (parent_adapter.empty()) {
                return {-1, "cannot create ipvlan network without parent"};
            }
            if (cidr_subnet.empty() || gateway.empty()) {
                const auto system_api =
                    dynamic_cast<const module_system_t*>(api::query_module("system").get());
                const auto adapters = system_api->get_network_adapters();
                const auto netif = adapters.find(parent_adapter.data());
                if (netif == adapters.cend()) {
                    return {-1, "network adapter does not exist"};
                }
                if (netif->second.ipv4_addr.empty()) {
                    return {-1, "network adapter is not ready"};
                }

                // create ipvlan network, if not exists
                subnet = ipv4_to_network(
                    netif->second.ipv4_addr[0].addr,
                    netif->second.ipv4_addr[0].subnet_mask);
                gw = netif->second.gateway;
            }
            break;
        }
        case network_type_e::MACVLAN: {
            break;
        }
        case network_type_e::Internal: {
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
    if (!parent_adapter.empty()) {
        docker_process.arg("--opt");
        docker_process.arg("parent=" + std::string{parent_adapter});
    }
    docker_process.arg(std::string{network});

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
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
        if (docker_process.exit_code() != 0) {
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
        if (docker_process.exit_code() != 0) {
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
        if (docker_process.exit_code() != 0) {
            return {};
        }
        auto out = docker_process.stdout();
        res.gateway = trim(out);
    }

    return res;
}

auto deployment_docker_t::do_delete_network(std::string_view network) //
    -> result_t
{
    auto docker_process = process_t{};

    docker_process.arg("network");
    docker_process.arg("rm");
    docker_process.arg(network.data());

    docker_process.spawnp("docker");
    docker_process.wait(false, false);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_connect_network(
    std::shared_ptr<instance_t> instance,
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
    docker_process.arg("flecs-" + instance->id().hex());

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_disconnect_network(
    std::shared_ptr<instance_t> instance, std::string_view network) //
    -> result_t
{
    auto docker_process = process_t{};

    docker_process.arg("network");
    docker_process.arg("disconnect");
    docker_process.arg("--force");
    docker_process.arg(std::string{network});
    docker_process.arg("flecs-" + instance->id().hex());

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_create_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "-" + std::string{volume_name};

    auto docker_process = process_t{};

    docker_process.arg("volume");
    docker_process.arg("create");
    docker_process.arg(name);

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto deployment_docker_t::do_import_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name, fs::path src_dir) //
    -> result_t
{
    using std::operator""s;

    const auto name = "flecs-"s + instance->id().hex() + "-"s + volume_name.data();
    const auto archive = src_dir.string() + "/" + name + "tar.gz";

    auto ec = std::error_code{};
    if (!fs::exists(archive, ec)) {
        return {-1, "Backup archive does not exist"};
    }
    if (!fs::is_regular_file(archive, ec)) {
        return {-1, "Backup archive is no regular file"};
    }

    delete_volume(instance, volume_name);
    create_volume(instance, volume_name);

    auto docker_process = process_t{};
    docker_process.arg("run");
    docker_process.arg("--rm");
    docker_process.arg("--volume");
    docker_process.arg(name + ":/mnt/restore:rw");
    docker_process.arg("--volume");
    docker_process.arg(src_dir.string() + ":"s + src_dir.string());
    docker_process.arg("--workdir");
    docker_process.arg(src_dir.string());
    docker_process.arg("alpine");
    docker_process.arg("tar");
    docker_process.arg("-C");
    docker_process.arg("/mnt/restore");
    docker_process.arg("-xf");
    docker_process.arg(name + ".tar.gz");
    docker_process.arg(".");
    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_export_volume(
    std::shared_ptr<instance_t> instance,
    std::string_view volume_name,
    fs::path dest_dir) const //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "-" + volume_name.data();

    auto docker_process = process_t{};

    docker_process.arg("run");
    docker_process.arg("--rm");
    docker_process.arg("--network");
    docker_process.arg("none");
    docker_process.arg("--volume");
    docker_process.arg(name + ":/mnt/backup:ro");
    docker_process.arg("--volume");
    docker_process.arg(dest_dir.string() + ":" + dest_dir.string());
    docker_process.arg("--workdir");
    docker_process.arg(dest_dir.string());
    docker_process.arg("alpine");
    docker_process.arg("tar");
    docker_process.arg("-C");
    docker_process.arg("/mnt/backup");
    docker_process.arg("-czf");
    docker_process.arg(name + ".tar.gz");
    docker_process.arg(".");
    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, {}};
}

auto deployment_docker_t::do_delete_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    auto docker_process = process_t{};

    const auto name = "flecs-" + instance->id().hex() + "-" + volume_name.data();

    docker_process.spawnp("docker", "volume", "rm", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, "Could not remove volume"};
    }

    return {0, ""};
}

auto deployment_docker_t::do_copy_file_from_image(
    std::string_view image,
    fs::path file,
    fs::path dest) //
    -> result_t
{
    auto container_id = std::string{};
    {
        auto docker_process = process_t{};
        docker_process.spawnp("docker", "create", image.data());
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0) {
            return {-1, "Could not create container"};
        }
        container_id = docker_process.stdout();
        trim(container_id);
    }
    {
        auto docker_process = process_t{};
        docker_process.spawnp("docker", "cp", container_id + ":" + file.string(), dest);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0) {
            return {-1, "Could not copy file from container"};
        }
    }
    {
        auto docker_process = process_t{};
        docker_process.spawnp("docker", "rm", "-f", container_id);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0) {
            return {-1, "Could not remove container"};
        }
    }

    return {0, {}};
}

auto deployment_docker_t::do_copy_file_to_instance(
    std::shared_ptr<instance_t> instance,
    fs::path file,
    fs::path dest) //
    -> result_t
{
    auto docker_process = process_t{};
    docker_process.arg("cp");
    docker_process.arg(file.string());
    docker_process.arg("flecs-" + instance->id().hex() + ":" + dest.string());
    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        using std::operator""s;
        return {
            -1,
            "Could not copy "s.append(file)
                .append(" to ")
                .append(instance->id().hex())
                .append(":")
                .append(dest)};
    }
    return {0, {}};
}

auto deployment_docker_t::do_copy_file_from_instance(
    std::shared_ptr<instance_t> instance,
    fs::path file,
    fs::path dest) const //
    -> result_t
{
    auto docker_process = process_t{};
    docker_process.arg("cp");
    docker_process.arg("flecs-" + instance->id().hex() + ":" + file.string());
    docker_process.arg(dest.string());
    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        using std::operator""s;
        return {
            -1,
            "Could not copy "s.append(instance->id().hex())
                .append(":")
                .append(file)
                .append(" to ")
                .append(file)};
    }
    return {0, {}};
}

auto deployment_docker_t::do_default_network_name() const //
    -> std::string_view
{
    return "flecs";
}

auto deployment_docker_t::do_default_network_type() const //
    -> network_type_e
{
    return network_type_e::Bridge;
}

auto deployment_docker_t::do_default_network_cidr_subnet() const //
    -> std::string_view
{
    return "172.21.0.0/16";
}

auto deployment_docker_t::do_default_network_gateway() const //
    -> std::string_view
{
    return "172.21.0.1";
}

} // namespace FLECS
