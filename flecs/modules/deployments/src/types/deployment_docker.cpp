// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "flecs/modules/deployments/types/deployment_docker.h"

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/util/cxx23/string.h"
#include "flecs/util/network/network.h"
#include "flecs/util/process/process.h"
#include "flecs/util/string/string_utils.h"
#include "flecs/util/sysfs/sysfs.h"

namespace flecs {
namespace deployments {

static const std::set<std::string> valid_capabilities = {
    "NET_ADMIN",
    "SYS_NICE",
    "IPC_LOCK",
    "NET_RAW",
};

auto docker_t::create_container(std::shared_ptr<instances::instance_t> instance) //
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
            if (cxx23::contains(docker_process.stdout(), container_name.c_str())) {
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

    auto environment = instance->environment();
    if (environment.has_value()) {
        for (const auto& env : environment.value()) {
            docker_process.arg("--env");
            docker_process.arg(stringify(env));
        }
    }

    for (const auto& volume : manifest->volumes()) {
        docker_process.arg("--volume");
        if (volume.type() == volume_t::BIND_MOUNT) {
            docker_process.arg(volume.host() + ":" + volume.container());
        } else {
            docker_process.arg(container_name + "-" + volume.host() + ":" + volume.container());
        }
    }
    auto ports = instance->ports();
    if (ports.has_value()) {
        for (const auto& port_range : ports.value()) {
            docker_process.arg("--publish");
            docker_process.arg(stringify(port_range));
        }
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

    for (const auto& label : manifest->labels()) {
        docker_process.arg("--label");
        docker_process.arg(to_string(label));
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

            if (network.mac_address.starts_with("clone:")) {
                const auto parts = split(network.mac_address, ':');
                if (parts.size() != 2) {
                    return {-1, "Cloned MAC address is invalid"};
                }

                const auto adapters = get_network_adapters();
                const auto netif = adapters.find(parts[1]);
                if (netif == adapters.cend()) {
                    return {-1, "Could not find network adapter for cloned MAC address"};
                }
                network.mac_address = std::string{netif->second.mac};
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

    for (const auto& cap : manifest->capabilities()) {
        if (valid_capabilities.contains(cap)) {
            docker_process.arg("--cap-add");
            docker_process.arg(cap);
        } else if (cap == "DOCKER") {
            docker_process.arg("--volume");
            docker_process.arg("/run/docker.sock:/run/docker.sock");
        }
    }

    if (std::find(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            (unsigned int)startup_option_t::INIT_NETWORK_AFTER_START) != instance->startup_options().cend()) {
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
        if (cmd.starts_with("/bin/sh -c ")) {
            cmd.erase(0, 11);
        }

        const auto entrypoint_path =
            std::string{"/var/lib/flecs/instances/"} + instance->id().hex() + std::string{"/scripts/"};

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
        const auto entrypoint_path = std::string{"/var/lib/flecs/instances/"} + instance->id().hex() +
                                     std::string{"/scripts/entrypoint.sh"};

        auto ec = std::error_code{};

        fs::permissions(
            entrypoint_path,
            fs::perms::owner_exec | fs::perms::group_exec | fs::perms::others_exec,
            ec);

        if (ec) {
            return {-1, "Could not make entrypoint executable"};
        }

        const auto [res, err_msg] = copy_file_to_instance(instance, entrypoint_path, "/flecs-entrypoint.sh");
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
                std::underlying_type_t<startup_option_t>(startup_option_t::INIT_NETWORK_AFTER_START)) ==
            instance->startup_options().cend()) {
            connect_network(instance, net->name, network.ip_address);
        }
    }

    return {0, {}};
}

auto docker_t::delete_container(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (app) {
        auto manifest = app->manifest();
        if (manifest) {
            const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
            for (const auto& conffile : manifest->conffiles()) {
                copy_file_from_instance(instance, conffile.container(), conf_path + conffile.local());
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

auto docker_t::do_deployment_id() const noexcept //
    -> std::string_view
{
    return "docker";
}

auto docker_t::do_download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
    -> result_t
{
    if (token.has_value()) {
        auto login_attempts = 3;
        while (login_attempts-- > 0) {
            auto process = process_t{};
            process.spawnp(
                "docker",
                "login",
                "--username",
                token->username.c_str(),
                "--password",
                token->password.c_str(),
                app->manifest()->image_with_tag());
            process.wait(true, true);
            if (process.exit_code() == 0) {
                break;
            }
        }
    }

    auto pull_process = process_t{};
    auto pull_attempts = 3;
    while (pull_attempts-- > 0) {
        pull_process.spawnp("docker", "pull", app->manifest()->image_with_tag());
        pull_process.wait(true, true);
        if (pull_process.exit_code() == 0) {
            break;
        }
    }

    if (token.has_value()) {
        auto process = process_t{};
        process.spawnp("docker", "logout");
        process.wait(true, true);
    }

    if (pull_process.exit_code() == 0) {
        return {0, {}};
    }

    return {-1, pull_process.stderr()};
}

auto docker_t::do_delete_app(std::shared_ptr<apps::app_t> app) //
    -> result_t
{
    const auto image = app->manifest()->image_with_tag();
    auto process = process_t{};
    process.spawnp("docker", "rmi", "-f", image);
    process.wait(false, true);
    if (process.exit_code() != 0) {
        return {-1, process.stderr()};
    }

    return {0, {}};
}

auto docker_t::do_import_app(std::shared_ptr<apps::app_t> /* app*/, fs::path archive) //
    -> result_t
{
    auto process = process_t{};
    process.spawnp("docker", "load", "--input", archive.c_str());
    process.wait(false, true);
    if (process.exit_code() != 0) {
        return {-1, process.stderr()};
    }

    return {0, {}};
}

auto docker_t::do_export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
    -> result_t
{
    auto process = process_t{};
    process.spawnp("docker", "save", "--output", archive.string(), app->manifest()->image_with_tag());
    process.wait(false, true);
    if (process.exit_code() != 0) {
        return {-1, process.stderr()};
    }

    return {0, {}};
}

auto docker_t::do_determine_app_size(std::shared_ptr<const apps::app_t> app) const //
    -> std::optional<std::size_t>
{
    auto process = process_t{};
    process.spawnp("docker", "inspect", "-f", "{{ .Size }}", app->manifest()->image_with_tag());
    process.wait(false, true);

    if (process.exit_code() == 0) {
        try {
            auto image_size = std::stoll(process.stdout());
            return image_size;
        } catch (...) {
        }
    }

    return {};
}

auto docker_t::do_create_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    instance->status(instances::status_e::Created);
    return {0, instance->id().hex()};
}

auto docker_t::do_delete_instance(std::shared_ptr<instances::instance_t> /*instance*/) //
    -> result_t
{
    return {0, ""};
}

auto docker_t::do_start_instance(std::shared_ptr<instances::instance_t> instance) //
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

auto docker_t::do_ready_instance(std::shared_ptr<instances::instance_t> instance) //
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

auto docker_t::do_stop_instance(std::shared_ptr<instances::instance_t> instance) //
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

auto docker_t::do_export_instance(
    std::shared_ptr<instances::instance_t> /*instance*/, fs::path /*dest_dir*/) const //
    -> result_t
{
    return {0, {}};
}

auto docker_t::do_import_instance(
    std::shared_ptr<instances::instance_t> /*instance*/, fs::path /*base_dir*/) //
    -> result_t
{
    return {0, {}};
}

auto docker_t::do_is_instance_running(std::shared_ptr<instances::instance_t> instance) const //
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

auto docker_t::do_networks() const //
    -> std::vector<network_t>
{
    auto docker_process = process_t{};
    docker_process.spawnp(
        "docker",       //
        "network",      //
        "ls",           //
        "--filter",     //
        "name=flecs.*", //
        "--format",
        "{{.Name}}");
    docker_process.wait(false, true);

    auto res = std::vector<network_t>{};
    if (docker_process.exit_code() == 0) {
        const auto networks = split(docker_process.stdout(), '\n');
        for (auto net : networks) {
            auto tmp = query_network(trim(net));
            if (tmp.has_value()) {
                res.push_back(std::move(*tmp));
            }
        }
    }

    return res;
}

auto docker_t::do_create_network(
    network_type_e network_type,
    std::string network_name,
    std::string cidr_subnet,
    std::string gateway,
    std::string parent_adapter) //
    -> result_t
{
    auto docker_process = process_t{};
    docker_process.arg("network");
    docker_process.arg("create");

    switch (network_type) {
        case network_type_e::Bridge:
        case network_type_e::MACVLAN:
        case network_type_e::Internal: {
            docker_process.arg("--driver");
            docker_process.arg(stringify(network_type));
            break;
        }

        case network_type_e::IPVLAN_L2:
        case network_type_e::IPVLAN_L3: {
            if (parent_adapter.empty()) {
                return {-1, "cannot create ipvlan network without parent"};
            }
            if (cidr_subnet.empty() || gateway.empty()) {
                const auto adapters = get_network_adapters();
                const auto netif = adapters.find(parent_adapter);
                if (netif == adapters.cend()) {
                    return {-1, "network adapter does not exist"};
                }
                if (netif->second.ipv4addresses.empty()) {
                    return {-1, "network adapter is not ready"};
                }

                cidr_subnet = ipv4_to_network(
                    netif->second.ipv4addresses[0].addr.data(),
                    netif->second.ipv4addresses[0].subnet_mask.data());
                gateway = std::string{netif->second.gateway};
            }

            docker_process.arg("--driver");
            docker_process.arg("ipvlan");
            docker_process.arg("--opt");
            docker_process.arg(
                std::string{"ipvlan_mode="} + (network_type == network_type_e::IPVLAN_L2 ? "l2" : "l3"));

            break;
        }
        default: {
            return {-1, "Invalid network_type specified"};
        }
    }

    docker_process.arg("--subnet");
    docker_process.arg(std::move(cidr_subnet));
    docker_process.arg("--gateway");
    docker_process.arg(std::move(gateway));
    if (!parent_adapter.empty()) {
        docker_process.arg("--opt");
        docker_process.arg("parent=" + std::move(parent_adapter));
    }
    docker_process.arg(std::move(network_name));

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        return {-1, docker_process.stderr()};
    }

    return {0, ""};
}

auto docker_t::do_query_network(std::string_view network) const //
    -> std::optional<network_t>
{
    auto res = network_t{};
    res.name = network;
    {
        // Get type of network
        auto docker_process = process_t{};
        docker_process.spawnp(
            "docker",
            "network",
            "inspect",
            "--format",
            "{{.Driver}}{{if ne .Options.ipvlan_mode nil}}_{{.Options.ipvlan_mode}}{{end}}",
            std::string{network});
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0) {
            return {};
        }

        auto network_type = docker_process.stdout();
        trim(network_type);
        res.type = network_type_from_string(network_type);
    }
    {
        // Get base IP and subnet of network as "a.b.c.d/x"
        auto docker_process = process_t{};
        docker_process.spawnp(
            "docker",
            "network",
            "inspect",
            "--format",
            "{{range .IPAM.Config}}{{.Subnet}}{{end}}",
            std::string{network});
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
        docker_process.spawnp(
            "docker",
            "network",
            "inspect",
            "--format",
            "{{range .IPAM.Config}}{{.Gateway}}{{end}}",
            std::string{network});
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0) {
            return {};
        }
        auto out = docker_process.stdout();
        res.gateway = trim(out);
    }
    {
        // Get parent adapter of network, if present
        auto docker_process = process_t{};
        docker_process.spawnp(
            "docker",
            "network",
            "inspect",
            "--format",
            "{{if ne .Options.parent nil}}{{.Options.parent}}{{end}}",
            std::string{network});
        docker_process.wait(false, false);
        if (docker_process.exit_code() != 0) {
            return {};
        }
        auto parent = docker_process.stdout();
        res.parent = trim(parent);
    }

    return res;
}

auto docker_t::do_delete_network(std::string_view network) //
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

auto docker_t::do_connect_network(
    std::shared_ptr<instances::instance_t> instance,
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

auto docker_t::do_disconnect_network(
    std::shared_ptr<instances::instance_t> instance, std::string_view network) //
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

auto docker_t::do_create_volume(
    std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
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

auto docker_t::do_import_volume(
    std::shared_ptr<instances::instance_t> instance, volume_t& volume, fs::path src_dir) //
    -> result_t
{
    using std::operator""s;

    const auto name = "flecs-"s + instance->id().hex() + "-"s + volume.host();
    const auto archive = src_dir.string() + "/" + name + ".tar.gz";

    auto ec = std::error_code{};
    if (!fs::exists(archive, ec)) {
        return {-1, "Backup archive does not exist"};
    }
    if (!fs::is_regular_file(archive, ec)) {
        return {-1, "Backup archive is no regular file"};
    }

    delete_volume(instance, volume.host());
    create_volume(instance, volume.host());

    auto docker_create_process = process_t{};
    docker_create_process.arg("create");
    docker_create_process.arg("--network");
    docker_create_process.arg("none");
    docker_create_process.arg("--volume");
    docker_create_process.arg(name + ":/mnt/restore:rw");
    docker_create_process.arg("--workdir");
    docker_create_process.arg("/mnt/restore");
    docker_create_process.arg("alpine");
    docker_create_process.arg("tar");
    docker_create_process.arg("-xf");
    docker_create_process.arg("/tmp/" + name + ".tar.gz");
    docker_create_process.spawnp("docker");
    docker_create_process.wait(false, true);
    if (docker_create_process.exit_code() != 0) {
        return {-1, docker_create_process.stderr()};
    }
    auto container_id = *split(docker_create_process.stdout(), '\n').rbegin();
    trim(container_id);

    auto docker_cp_process = process_t{};
    docker_cp_process.arg("cp");
    docker_cp_process.arg(archive);
    docker_cp_process.arg(container_id + ":/tmp/");
    docker_cp_process.spawnp("docker");
    docker_cp_process.wait(false, true);
    if (docker_cp_process.exit_code() != 0) {
        return {-1, docker_cp_process.stderr()};
    }

    auto docker_start_process = process_t{};
    docker_start_process.arg("start");
    docker_start_process.arg(container_id);
    docker_start_process.spawnp("docker");
    docker_start_process.wait(false, true);
    if (docker_start_process.exit_code() != 0) {
        return {-1, docker_start_process.stderr()};
    }

    auto docker_rm_process = process_t{};
    docker_rm_process.arg("rm");
    docker_rm_process.arg("--force");
    docker_rm_process.arg(container_id);
    docker_rm_process.spawnp("docker");
    docker_rm_process.wait(false, true);

    return {0, {}};
}

auto docker_t::do_export_volume(
    std::shared_ptr<instances::instance_t> instance,
    const volume_t& volume,
    fs::path dest_dir) const //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "-" + volume.host();
    const auto archive = name + ".tar.gz";

    auto docker_create_process = process_t{};
    docker_create_process.arg("create");
    docker_create_process.arg("--network");
    docker_create_process.arg("none");
    docker_create_process.arg("--volume");
    docker_create_process.arg(name + ":/mnt/backup:ro");
    docker_create_process.arg("--workdir");
    docker_create_process.arg("/tmp");
    docker_create_process.arg("alpine");
    docker_create_process.arg("tar");
    docker_create_process.arg("-C");
    docker_create_process.arg("/mnt/backup");
    docker_create_process.arg("-czf");
    docker_create_process.arg(name + ".tar.gz");
    docker_create_process.arg(".");
    docker_create_process.spawnp("docker");
    docker_create_process.wait(false, true);
    if (docker_create_process.exit_code() != 0) {
        return {-1, docker_create_process.stderr()};
    }
    auto container_id = *split(docker_create_process.stdout(), '\n').rbegin();
    trim(container_id);

    auto docker_start_process = process_t{};
    docker_start_process.arg("start");
    docker_start_process.arg(container_id);
    docker_start_process.spawnp("docker");
    docker_start_process.wait(false, true);
    if (docker_start_process.exit_code() != 0) {
        return {-1, docker_start_process.stderr()};
    }

    auto docker_cp_process = process_t{};
    docker_cp_process.arg("cp");
    docker_cp_process.arg(container_id + ":/tmp/" + archive);
    docker_cp_process.arg(dest_dir);
    docker_cp_process.spawnp("docker");
    docker_cp_process.wait(false, true);
    if (docker_cp_process.exit_code() != 0) {
        return {-1, docker_cp_process.stderr()};
    }

    auto docker_rm_process = process_t{};
    docker_rm_process.arg("rm");
    docker_rm_process.arg("--force");
    docker_rm_process.arg(container_id);
    docker_rm_process.spawnp("docker");
    docker_rm_process.wait(false, true);

    return {0, {}};
}

auto docker_t::do_delete_volume(
    std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
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

auto docker_t::do_copy_file_from_image(
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

auto docker_t::do_copy_file_to_instance(
    std::shared_ptr<instances::instance_t> instance,
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

auto docker_t::do_copy_file_from_instance(
    std::shared_ptr<instances::instance_t> instance,
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

auto docker_t::do_default_network_name() const //
    -> std::string_view
{
    return "flecs";
}

auto docker_t::do_default_network_type() const //
    -> network_type_e
{
    return network_type_e::Bridge;
}

auto docker_t::do_default_network_cidr_subnet() const //
    -> std::string_view
{
    return "172.21.0.0/16";
}

auto docker_t::do_default_network_gateway() const //
    -> std::string_view
{
    return "172.21.0.1";
}
docker_t::docker_t() = default;

docker_t::~docker_t() = default;

} // namespace deployments
} // namespace flecs
