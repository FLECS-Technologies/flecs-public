#include "instance/instance_config.h"

#include <arpa/inet.h>
#include <netinet/in.h>

#include <bitset>

#include "factory/factory.h"
#include "private/app_manager_private.h"
#include "system/system.h"
#include "util/network/network.h"
#include "util/process/process.h"
#include "util/usb/usb.h"

namespace FLECS {
namespace Private {

namespace {
auto build_network_adapters_json(const instance_t& instance)
{
    const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
    const auto adapters = system_api->get_network_adapters();
    auto adapters_json = json_t::array();
    for (decltype(auto) adapter : adapters)
    {
        if ((adapter.second.type == netif_type_t::WIRED) || (adapter.second.type == netif_type_t::WIRELESS))
        {
            auto adapter_json = json_t{};
            adapter_json["name"] = adapter.first;
            adapter_json["active"] = false;
            auto network = std::string{"flecs-macvlan-"} + adapter.first;
            auto it = std::find_if(
                instance.networks().cbegin(),
                instance.networks().cend(),
                [&](const instance_t::network_t& n) { return n.network_name == network; });
            if (it != instance.networks().cend())
            {
                adapter_json["active"] = true;
                adapter_json["ipAddress"] = it->ip_address;
                adapter_json["subnetMask"] = adapter.second.ipv4_addr.begin()->subnet_mask;
                adapter_json["gateway"] = adapter.second.gateway;
            }
            adapters_json.push_back(adapter_json);
        }
    }
    return adapters_json;
}
} // namespace

auto module_app_manager_private_t::do_post_config_instance(const std::string& instance_id, json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instance_id;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id))
    {
        response["additionalInfo"] = "Could not configure instance " + instance_id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    const auto& instance = _deployment->instances().at(instance_id);
    response["networkAdapters"] = build_network_adapters_json(instance);

    const auto usb_devices = usb::get_devices();
    response["devices"] = FLECS::json_t::object();
    response["devices"]["usb"] = json_t(usb_devices);

    return crow::status::OK;
}

auto module_app_manager_private_t::do_put_config_instance(
    const std::string& instance_id,
    const instance_config_t& config,
    json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instance_id;

    // Step 1: Verify instance does actually exist
    if (!_deployment->has_instance(instance_id))
    {
        response["additionalInfo"] = "Could not configure instance " + instance_id + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    auto& instance = _deployment->instances().at(instance_id);
    response["networkAdapters"] = build_network_adapters_json(instance);

    const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
    const auto adapters = system_api->get_network_adapters();

    for (const auto& network : config.networkAdapters)
    {
        const auto docker_network = std::string{"flecs-macvlan-"} + network.name;
        if (network.active)
        {
            // ensure network adapter exists
            const auto netif = adapters.find(network.name);
            if (netif == adapters.cend())
            {
                continue;
            }
            if (netif->second.ipv4_addr.empty())
            {
                response["additionalInfo"] = "Network adapter " + netif->first + " not ready";
                continue;
            }

            // create macvlan network, if not exists
            const auto cidr_subnet =
                ipv4_to_network(netif->second.ipv4_addr[0].addr, netif->second.ipv4_addr[0].subnet_mask);
            _deployment->create_network(
                network_type_t::MACVLAN,
                docker_network,
                cidr_subnet,
                netif->second.gateway,
                netif->first);

            // process instance configuration
            if (network.ipAddress.empty())
            {
                // suggest suitable IP address
                for (auto& adapter_json : response["networkAdapters"])
                {
                    if (adapter_json["name"] == netif->first)
                    {
                        adapter_json["active"] = true;
                        adapter_json["ipAddress"] = generate_ip(cidr_subnet);
                        adapter_json["subnetMask"] = netif->second.ipv4_addr[0].subnet_mask;
                        adapter_json["gateway"] = netif->second.gateway;
                        break;
                    }
                }
            }
            else
            {
                // apply settings
                // @todo verify validity of IP address
                _deployment->disconnect_network(instance_id, docker_network);

                const auto [res, additional_info] =
                    _deployment->connect_network(instance_id, docker_network, network.ipAddress);

                if (res == 0)
                {
                    auto it = std::find_if(
                        instance.networks().begin(),
                        instance.networks().end(),
                        [&](const instance_t::network_t& n) { return n.network_name == docker_network; });
                    if (it != instance.networks().end())
                    {
                        it->ip_address = network.ipAddress;
                    }
                    else
                    {
                        instance.networks().emplace_back(
                            instance_t::network_t{.network_name = docker_network, .ip_address = network.ipAddress});
                    }
                    const auto it2 = std::find_if(
                        instance.config().networkAdapters.begin(),
                        instance.config().networkAdapters.end(),
                        [&](const instance_config_t::network_adapter_t& adapter) {
                            return adapter.name == network.name;
                        });
                    if (it2 != instance.config().networkAdapters.end())
                    {
                        *it2 = network;
                    }
                    else
                    {
                        instance.config().networkAdapters.emplace_back(network);
                    }
                    persist_instances();
                    for (auto& adapter_json : response["networkAdapters"])
                    {
                        if (adapter_json.contains("name") && (adapter_json["name"] == netif->first))
                        {
                            adapter_json["active"] = true;
                            adapter_json["ipAddress"] = network.ipAddress;
                        }
                    }
                }
                else
                {
                    response["additionalInfo"] = additional_info;
                    for (auto& adapter_json : response["networkAdapters"])
                    {
                        if (adapter_json["name"] == netif->first)
                        {
                            adapter_json["active"] = false;
                        }
                    }
                }
            }
        }
        else
        {
            _deployment->disconnect_network(instance_id, docker_network);
            _deployment->delete_network(docker_network);

            instance.networks().erase(
                std::remove_if(
                    instance.networks().begin(),
                    instance.networks().end(),
                    [&](const instance_t::network_t& net) { return net.network_name == docker_network; }),
                instance.networks().end());

            instance.config().networkAdapters.erase(
                std::remove_if(
                    instance.config().networkAdapters.begin(),
                    instance.config().networkAdapters.end(),
                    [&](const instance_config_t::network_adapter_t& adapter) { return adapter.name == network.name; }),
                instance.config().networkAdapters.end());

            for (auto& adapter_json : response["networkAdapters"])
            {
                if (adapter_json.contains("name") && (adapter_json["name"] == network.name))
                {
                    adapter_json["active"] = false;
                }
            }
        }
    }
    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
