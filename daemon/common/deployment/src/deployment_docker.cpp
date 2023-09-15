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

#include "deployment_docker.h"

#include "app/app.h"
#include "app/manifest/manifest.h"
#include "factory/factory.h"
#include "system/system.h"
#include "util/cxx20/string.h"
#include "util/docker/libdocker_api_client.h"
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
    auto client = setup_libdocker_client();

    {
        auto containers = client.list_containers(true, 0, false);
        auto container_it = std::find_if(
            containers.cbegin(),
            containers.cend(),
            [container_name](const auto& elem) {
                return elem.image_name() == container_name.c_str();
            });
        auto container_exists = container_it != containers.cend();
        if (container_exists) {
            return {0, "Container already exists"};
        }
    }

    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    // check if image exists
    auto im = client.inspect_image(manifest->image_with_tag());
    if (!im.has_value()) {
        auto image_name = manifest->image();
        auto image_version = manifest->version();
        auto j = json_t({{"fromImage", image_name}, {"tag", image_version}});
        client.create_image(j);
    }

    /* See https://docs.docker.com/engine/api/v1.41/#tag/Container/operation/ContainerCreate */
    auto create_args = json_t();

    for (const auto& env : manifest->env()) {
        create_args["Env"].push_back(stringify(env));
    }
    for (const auto& volume : manifest->volumes()) {
        create_args["Volumes"][volume.container()] = json_t::object();
        create_args["HostConfig"]["Binds"].push_back(volume.host() + ":" + volume.container());
    }
    for (const auto& port_range : manifest->ports()) {
        auto container_range = port_range.container_port_range();
        auto host_range = port_range.host_port_range();
        auto port_count = container_range.end_port() - container_range.start_port() + 1;
        for (auto i = 0; i < port_count; i++) {
            auto container_port = container_range.start_port() + i;
            auto host_port = host_range.start_port() + i;
            //
            create_args["ExposedPorts"][stringify(host_port)] = json_t::object();
            create_args["HostConfig"]["PortBindings"][stringify(container_port)] = json_t::array(
                {json_t::object({{"HostPort", stringify(host_port)}})}); // really, docker engine?
        }
    }

    if (!manifest->hostname().empty()) {
        create_args["Hostname"] = manifest->hostname();
    } else {
        create_args["Hostname"] = container_name;
    }

    for (const auto& device : manifest->devices()) {
        create_args["HostConfig"]["Devices"].push_back(json_t::object(
            {{"PathInContainer", device},
             {"PathOnHost", device},
             {"CgroupPermissions",
              "none"}})); /** @todo what are the options for CgroupPermissions? */
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

        create_args["HostConfig"]["NetworkMode"] = instance->networks()[0].network_name;
        create_args["HostConfig"]["NetworkingConfig"]["EndpointsConfig"]
                   [instance->networks()[0].network_name] =
                       json_t::object({{"IPAddress", network.ip_address}});

        if (!network.mac_address.empty()) {
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
            create_args["MacAddress"] = network.mac_address;
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
                create_args["HostConfig"]["Devices"].push_back(json_t::object(
                    {{"PathInContainer", path},
                     {"PathOnHost", path},
                     {"CgroupPermissions",
                      "none"}})); /** @todo what are the options for CgroupPermissions? */
            }
        }
    }

    for (const auto& cap : manifest->capabilities()) {
        if (cap == "NET_ADMIN") {
            create_args["HostConfig"]["CapAdd"].push_back(cap);
        } else if (cap == "DOCKER") {
            create_args["Volumes"]["/run/docker.sock"] = json_t::object();
            create_args["HostConfig"]["Binds"].push_back("/run/docker.sock:/run/docker.sock");
        }
    }

    if (std::find(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            (unsigned int)startup_option_t::INIT_NETWORK_AFTER_START) !=
        instance->startup_options().cend()) {
        create_args["HostConfig"]["Mounts"].push_back(
            {{"Type", "tmpfs"}, {"Target", "/flecs-tmp"}});
        auto cmd = std::string{};
        {
            auto img = client.inspect_image(manifest->image_with_tag());
            auto config = img->config();
            if (config.count("Cmd")) {
                auto cmd = config["Cmd"];
            } else {
                return {-1, "Could not determine entrypoint"};
            }
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

        create_args["Entrypoint"] = json_t::array({"/flecs-entrypoint.sh"});
    }

    create_args["Image"] = manifest->image_with_tag();

    for (const auto& arg : manifest->args()) {
        /** @todo what kinds of arguments are these? Where do they go in the JSON? */
        // docker_process.arg(arg);
    }

    auto [code, create_response] = client.create_container(create_args, container_name);

    if (code != 0) {
        return {code, "Could not create Docker container"};
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
} // namespace FLECS

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

    auto client = setup_libdocker_client();
    auto [code, response] = client.remove_container(container_name, false, true); // rm -f

    if (code != 0) {
        return {code, response};
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

    auto client = setup_libdocker_client();
    auto [code, response] = client.start_container(container_name);

    if (code != 0) {
        return {code, response};
    }

    return {0, {}};
}

auto deployment_docker_t::do_ready_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto container_name = "flecs-" + instance->id().hex();

    auto client = setup_libdocker_client();
    json_t args;
    args["Cmd"] = {"touch /flecs-tmp/ready"};

    auto [code, response] = client.exec(container_name, args);
    if (code != 0) {
        return {0, response};
    }

    return {0, {}};
}

auto deployment_docker_t::do_stop_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto container_name = "flecs-" + instance->id().hex();

    auto client = setup_libdocker_client();
    auto [code, response] = client.stop_container(container_name, 0);

    return delete_container(instance);
}

auto deployment_docker_t::do_export_instance(
    std::shared_ptr<instance_t> /*instance*/, fs::path /*dest_dir*/) const //
    -> result_t
{
    return {0, {}};
}

auto deployment_docker_t::do_import_instance(
    std::shared_ptr<instance_t> /*instance*/, fs::path /*base_dir*/) //
    -> result_t
{
    return {0, {}};
}

auto deployment_docker_t::do_is_instance_running(std::shared_ptr<instance_t> instance) const //
    -> bool
{
    auto container_name = "flecs-" + instance->id().hex();
    auto client = setup_libdocker_client();
    auto response = client.inspect_container(container_name, false);

    // Consider instance running if 'docker inspect' found the container
    return response.has_value();
}

auto deployment_docker_t::do_create_network(
    network_type_e network_type,
    std::string_view network,
    std::string_view cidr_subnet,
    std::string_view gateway,
    std::string_view parent_adapter) //
    -> result_t
{
    auto client = setup_libdocker_client();

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

    auto create_args = json_t();
    create_args["Name"] = network;
    create_args["Driver"] = stringify(network_type);
    create_args["IPAM"]["Config"] = {{"Subnet", subnet}, {"Gateway", gw}};

    /** @todo where to specify parent adapters? */
    /*
    if (!parent_adapter.empty()) {
        docker_process.arg("--opt");
        docker_process.arg("parent=" + std::string{parent_adapter});
    }
    docker_process.arg(std::string{network}); */

    auto [code, response] = client.create_network(create_args);

    if (code != 0) {
        return {code, response};
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
        auto client = setup_libdocker_client();
        auto response = client.inspect_network(network, true, "local");
        /** @todo Driver? */
        if (!response.has_value()) {
            return {};
        }

        res.type = network_type_from_string(response->name()); /** @todo network type */
    }
    {
        // Get base IP and subnet of network as "a.b.c.d/x"
        auto client = setup_libdocker_client();
        auto response = client.inspect_network(network, true, "local");
        if (!response.has_value()) {
            return {};
        }

        res.cidr_subnet = response->subnet();
    }
    {
        // Get gateway of network as "a.b.c.d"
        auto client = setup_libdocker_client();
        auto response = client.inspect_network(network, true, "local");
        if (!response.has_value()) {
            return {};
        }

        res.gateway = response->gateway();
    }

    return res;
}

auto deployment_docker_t::do_delete_network(std::string_view network) //
    -> result_t
{
    auto client = setup_libdocker_client();
    auto [code, response] = client.remove_network(network);

    if (code != 0) {
        return {code, response};
    }

    return {0, {}};
}

auto deployment_docker_t::do_connect_network(
    std::shared_ptr<instance_t> instance,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    auto client = setup_libdocker_client();

    auto container_name = "flecs-" + instance->id().hex();
    auto [code, response] =
        client.connect_container(network, container_name); /** @todo add IP option */

    if (code != 0) {
        return {code, response};
    }

    return {0, ""};
}

auto deployment_docker_t::do_disconnect_network(
    std::shared_ptr<instance_t> instance, std::string_view network) //
    -> result_t
{
    auto client = setup_libdocker_client();

    auto container_name = "flecs-" + instance->id().hex();
    auto [code, response] = client.disconnect_container(network, container_name, true);

    if (code != 0) {
        return {code, response};
    }

    return {0, ""};
}

auto deployment_docker_t::do_create_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    const auto name = "flecs-" + instance->id().hex() + "-" + std::string{volume_name};

    auto client = setup_libdocker_client();
    auto [code, response] = client.create_volume(name);

    if (code != 0) {
        return {code, response};
    }

    return {0, ""};
}

auto deployment_docker_t::do_import_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name, fs::path src_dir) //
    -> result_t
{
    using std::operator""s;

    const auto name = "flecs-"s + instance->id().hex() + "-"s + volume_name.data();
    const auto archive = src_dir.string() + "/" + name + ".tar.gz";

    auto ec = std::error_code{};
    if (!fs::exists(archive, ec)) {
        return {-1, "Backup archive does not exist"};
    }
    if (!fs::is_regular_file(archive, ec)) {
        return {-1, "Backup archive is no regular file"};
    }

    delete_volume(instance, volume_name);
    create_volume(instance, volume_name);

    auto client = setup_libdocker_client();

    auto create_args = json_t();
    // create_args["Volumes"][volume.container()] = json_t::object();
    create_args["NetworkDisabled"] = true;
    create_args["HostConfig"]["Binds"].push_back(name + ":/mnt/restore:rw");
    create_args["WorkingDir"] = "/mnt/restore";
    create_args["Image"] = "alpine";
    create_args["Cmd"] = "tar -xf /tmp/" + name + ".tar.gz";

    auto [code, response] = client.create_container("", "", to_string(create_args));
    if (code != 0) {
        return {code, response};
    }

    auto container_id = to_string(response.at("Id"));

    auto cp_client = setup_libdocker_client();
    auto [cp_code, cp_response] = cp_client.copy_to_container(container_id, "/tmp/", archive);
    if (cp_code != 0) {
        return {cp_code, cp_response};
    }

    auto start_client = setup_libdocker_client();
    auto [start_code, start_response] = start_client.start_container(container_id);
    if (start_code != 0) {
        return {start_code, start_response};
    }

    auto rm_client = setup_libdocker_client();
    auto [rm_code, rm_response] = rm_client.remove_container(container_id, false, true);
    if (rm_code != 0) {
        return {rm_code, rm_response};
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
    const auto archive = name + ".tar.gz";

    auto create_client = setup_libdocker_client();
    auto create_args = json_t();
    create_args["NetworkDisabled"] = true;
    create_args["Image"] = "alpine";
    create_args["Cmd"] = "tar -C /mnt/backup -czf " + archive + " . ";
    create_args["WorkingDir"] = "/tmp";
    // create_args["Volumes"] = json_t{{"/mnt/backup", {}}};
    create_args["HostConfig"]["Binds"].push_back(name + ":/mnt/backup:ro");

    auto [create_code, create_response] =
        create_client.create_container("", "", to_string(create_args));
    if (create_code != 0) {
        return {create_code, create_response};
    }

    auto container_id = to_string(create_response.at("Id"));

    auto start_client = setup_libdocker_client();
    auto [start_code, start_response] = start_client.start_container(container_id);
    if (start_code != 0) {
        return {start_code, start_response};
    }

    auto cp_client = setup_libdocker_client();
    auto [cp_code, cp_response] = cp_client.extract_from_container(container_id, "/tmp/", dest_dir);
    if (cp_code != 0) {
        return {cp_code, cp_response};
    }

    auto rm_client = setup_libdocker_client();
    auto [rm_code, rm_response] = rm_client.remove_container(container_id, false, true);
    if (rm_code != 0) {
        return {rm_code, rm_response};
    }

    return {0, {}};
}

auto deployment_docker_t::do_delete_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    auto client = setup_libdocker_client();
    const auto name = "flecs-" + instance->id().hex() + "-" + volume_name.data();

    auto [code, response] = client.remove_volume(name);
    if (code != 0) {
        return {code, "Could not remove volume"};
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
        auto client = setup_libdocker_client();
        auto create_args = json_t();
        create_args["Image"] = image;
        auto [code, response] = client.create_container(create_args, "");

        if (code != 0) {
            return {code, "Could not create container"};
        }
        container_id = response.at("Id");
        trim(container_id);
    }
    /** @todo docker cp */
    {
        auto client = setup_libdocker_client();
        auto [code, response] = client.extract_from_container(
            container_id,
            file.string(),
            dest); /** @todo writes archive instead of file */
        if (code != 0) {
            return {code, "Could not copy file from container"};
        }
    }
    {
        auto client = setup_libdocker_client();

        auto [code, response] =
            client.remove_container(container_id, false, true); /** @todo swap args */
        if (code != 0) {
            return {code, "Could not remove container"};
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
    auto client = setup_libdocker_client();
    auto name = "flecs-" + instance->id().hex();

    auto [code, response] = client.copy_to_container(
        name,
        dest.string(),
        file.string()); /** @todo reads archive instead of file */
    if (code != 0) {
        using std::operator""s;
        return {
            code,
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
    auto client = setup_libdocker_client();
    auto name = "flecs-" + instance->id().hex();

    auto [code, response] = client.extract_from_container(
        name,
        file.string(),
        dest.string()); /** @todo writes archive instead of file */
    if (!response.empty()) {
        using std::operator""s;
        return {
            code,
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
