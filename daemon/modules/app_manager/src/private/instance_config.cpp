#include "instance/instance_config.h"

#include <arpa/inet.h>
#include <netinet/in.h>

#include <bitset>

#include "factory/factory.h"
#include "private/app_manager_private.h"
#include "system/system.h"
#include "util/cxx20/string.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "util/usb/usb.h"

namespace FLECS {
namespace Private {
#if 0
namespace {
auto build_network_adapters_json(const instance_t& instance)
{
    const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
    const auto adapters = system_api->get_network_adapters();
    auto adapters_json = json_t::array();
    for (decltype(auto) adapter : adapters) {
        if ((adapter.second.type == netif_type_t::WIRED) ||
            (adapter.second.type == netif_type_t::WIRELESS)) {
            auto adapter_json = json_t{};
            adapter_json["name"] = adapter.first;
            adapter_json["active"] = false;
            adapter_json["connected"] = !adapter.second.ipv4_addr.empty();
            auto network = std::string{"flecs-macvlan-"} + adapter.first;
            auto it = std::find_if(
                instance.networks().cbegin(),
                instance.networks().cend(),
                [&](const instance_t::network_t& n) { return n.network_name == network; });
            if (it != instance.networks().cend()) {
                adapter_json["active"] = true;
                adapter_json["ipAddress"] = it->ip_address;
                if (adapter.second.ipv4_addr.empty()) {
                    adapter_json["subnetMask"] = "0.0.0.0";
                    adapter_json["gateway"] = "0.0.0.0";
                } else {
                    adapter_json["subnetMask"] = adapter.second.ipv4_addr.begin()->subnet_mask;
                    adapter_json["gateway"] = adapter.second.gateway;
                }
            }
            adapters_json.push_back(adapter_json);
        }
    }
    for (decltype(auto) network : instance.networks()) {
        if (cxx20::starts_with(network.network_name, "flecs-macvlan-")) {
            const auto adapter = network.network_name.substr(14);
            if (!adapters.count(adapter)) {
                auto adapter_json = json_t{};
                adapter_json["name"] = adapter;
                adapter_json["active"] = true;
                adapter_json["connected"] = false;
                adapter_json["ipAddress"] = network.ip_address;
                adapter_json["subnetMask"] = "0.0.0.0";
                adapter_json["gateway"] = "0.0.0.0";

                adapters_json.push_back(std::move(adapter_json));
            }
        }
    }
    return adapters_json;
}

auto build_usb_devices_json(const instance_t& instance)
{
    auto ret = json_t::array();
    const auto usb_devices = usb::get_devices();

    // insert connected usb device
    for (decltype(auto) usb_device : usb_devices) {
        auto device_json = json_t(usb_device);
        device_json["active"] = static_cast<bool>(instance.usb_devices().count(usb_device));
        device_json["connected"] = true;
        ret.push_back(std::move(device_json));
    }

    // insert configured, but disconnected usb devices
    for (decltype(auto) usb_device : instance.usb_devices()) {
        if (!usb_devices.count(usb_device)) {
            auto device_json = json_t(usb_device);
            device_json["active"] = true;
            device_json["connected"] = false;
            ret.push_back(std::move(device_json));
        }
    }

    return ret;
}
} // namespace
#endif // 0
auto module_app_manager_private_t::do_get_config_instance(
    const instance_id_t& /*instance_id*/, json_t& /*response*/) //
    -> crow::status
{
#if 0
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instance_id;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] =
            "Could not configure instance " + instance_id.hex() + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    const auto& instance = _deployment->instances().at(instance_id);
    response["networkAdapters"] = build_network_adapters_json(instance);
    response["devices"]["usb"] = build_usb_devices_json(instance);
#endif // 0
    return crow::status::OK;
}

auto module_app_manager_private_t::do_put_config_instance(
    const instance_id_t& /*instance_id*/,
    const instance_config_t& /*config*/,
    json_t& /*response*/) //
    -> crow::status
{
#if 0
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instance_id;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id)) {
        response["additionalInfo"] =
            "Could not configure instance " + instance_id.hex() + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    auto& instance = _deployment->instances().at(instance_id);
    response["networkAdapters"] = build_network_adapters_json(instance);

    const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
    const auto adapters = system_api->get_network_adapters();

    for (const auto& network : config.networkAdapters) {
        const auto docker_network = std::string{"flecs-macvlan-"} + network.name;
        if (network.active) {
            // ensure network adapter exists
            const auto netif = adapters.find(network.name);
            if (netif == adapters.cend()) {
                continue;
            }
            if (netif->second.ipv4_addr.empty()) {
                response["additionalInfo"] = "Network adapter " + netif->first + " not ready";
                continue;
            }

            // create macvlan network, if not exists
            const auto cidr_subnet = ipv4_to_network(
                netif->second.ipv4_addr[0].addr,
                netif->second.ipv4_addr[0].subnet_mask);

            // process instance configuration
            if (network.ipAddress.empty()) {
                // suggest suitable IP address
                for (auto& adapter_json : response["networkAdapters"]) {
                    if (adapter_json["name"] == netif->first) {
                        adapter_json["active"] = true;
                        adapter_json["ipAddress"] =
                            _deployment->generate_instance_ip(cidr_subnet, netif->second.gateway);
                        adapter_json["subnetMask"] = netif->second.ipv4_addr[0].subnet_mask;
                        adapter_json["gateway"] = netif->second.gateway;
                        break;
                    }
                }
            } else {
                // apply settings
                // @todo verify validity of IP address
                _deployment->create_network(
                    network_type_e::MACVLAN,
                    docker_network,
                    cidr_subnet,
                    netif->second.gateway,
                    netif->first);

                _deployment->disconnect_network(instance_id, docker_network);

                const auto [res, additional_info] =
                    _deployment->connect_network(instance_id, docker_network, network.ipAddress);

                if (res == 0) {
                    auto it = std::find_if(
                        instance.networks().begin(),
                        instance.networks().end(),
                        [&](const instance_t::network_t& n) {
                            return n.network_name == docker_network;
                        });
                    if (it != instance.networks().end()) {
                        it->ip_address = network.ipAddress;
                    } else {
                        instance.networks().emplace_back(instance_t::network_t{
                            .network_name = docker_network,
                            .mac_address = {},
                            .ip_address = network.ipAddress});
                    }
                    _deployment->save();
                    for (auto& adapter_json : response["networkAdapters"]) {
                        if (adapter_json.contains("name") &&
                            (adapter_json["name"] == netif->first)) {
                            adapter_json["active"] = true;
                            adapter_json["ipAddress"] = network.ipAddress;
                        }
                    }
                } else {
                    response["additionalInfo"] = additional_info;
                    for (auto& adapter_json : response["networkAdapters"]) {
                        if (adapter_json["name"] == netif->first) {
                            adapter_json["active"] = false;
                        }
                    }
                }
            }
        } else {
            _deployment->disconnect_network(instance_id, docker_network);
            _deployment->delete_network(docker_network);

            instance.networks().erase(
                std::remove_if(
                    instance.networks().begin(),
                    instance.networks().end(),
                    [&](const instance_t::network_t& net) {
                        return net.network_name == docker_network;
                    }),
                instance.networks().end());

            for (auto& adapter_json : response["networkAdapters"]) {
                if (adapter_json.contains("name") && (adapter_json["name"] == network.name)) {
                    adapter_json["active"] = false;
                }
            }
        }
    }

    for (const auto& usb_device : config.usb_devices) {
        if (usb_device.active) {
            instance.usb_devices().emplace(usb_device);
        } else {
            instance.usb_devices().erase(usb_device);
        }
    }
    response["devices"]["usb"] = build_usb_devices_json(instance);
#endif // 0
    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
