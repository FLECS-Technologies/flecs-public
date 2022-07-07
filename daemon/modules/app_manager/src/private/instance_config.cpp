#include "instance/instance_config.h"

#include <arpa/inet.h>
#include <netinet/in.h>

#include <bitset>

#include "factory/factory.h"
#include "private/app_manager_private.h"
#include "system/system.h"
#include "util/network/network.h"
#include "util/process/process.h"

namespace FLECS {
namespace Private {

namespace {
auto build_network_adapters_json(const instances_table_entry_t& instance)
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
            auto it = std::find(instance.networks.cbegin(), instance.networks.cend(), network);
            if (it != instance.networks.cend())
            {
                const auto pos = std::distance(instance.networks.cbegin(), it);
                adapter_json["active"] = true;
                adapter_json["ipAddress"] = instance.ips[pos];
                adapter_json["subnetMask"] = adapter.second.ipv4_addr.begin()->subnet_mask;
                adapter_json["gateway"] = adapter.second.gateway;
            }
            adapters_json.push_back(adapter_json);
        }
    }
    return adapters_json;
}
} // namespace

auto module_app_manager_private_t::do_post_config_instance(const std::string& instanceId, json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instanceId;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({instanceId}))
    {
        response["additionalInfo"] = "Could not configure instance " + instanceId + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    const auto instance = _app_db.query_instance({instanceId}).value();
    response["networkAdapters"] = build_network_adapters_json(instance);

    return crow::status::OK;
}

auto module_app_manager_private_t::do_put_config_instance(
    const std::string& instanceId,
    const instance_config_t& config,
    json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instanceId;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({instanceId}))
    {
        response["additionalInfo"] = "Could not configure instance " + instanceId + ", which does not exist";
        return crow::status::BAD_REQUEST;
    }

    auto instance = _app_db.query_instance({instanceId}).value();
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
                _deployment->disconnect_network(instanceId, docker_network);

                const auto [res, additional_info] =
                    _deployment->connect_network(instanceId, docker_network, network.ipAddress);

                if (res == 0)
                {
                    const auto it = std::find(instance.networks.cbegin(), instance.networks.cend(), docker_network);
                    if (it != instance.networks.end())
                    {
                        const auto pos = std::distance(instance.networks.cbegin(), it);
                        instance.ips[pos] = network.ipAddress;
                    }
                    else
                    {
                        instance.networks.emplace_back(docker_network);
                        instance.ips.emplace_back(network.ipAddress);
                    }
                    const auto it2 = std::find_if(
                        _deployment->instances().at(instanceId).config().networks.begin(),
                        _deployment->instances().at(instanceId).config().networks.end(),
                        [&](const instance_config_t::network_t& network) { return network.network == docker_network; });
                    if (it2 != _deployment->instances().at(instanceId).config().networks.end())
                    {
                        it2->ip = network.ipAddress;
                    }
                    else
                    {
                        _deployment->instances()
                            .at(instanceId)
                            .config()
                            .networks.emplace_back(
                                instance_config_t::network_t{.network = docker_network, .ip = network.ipAddress});
                    }
                    const auto it3 = std::find_if(
                        _deployment->instances().at(instanceId).config().networkAdapters.begin(),
                        _deployment->instances().at(instanceId).config().networkAdapters.end(),
                        [&](const instance_config_t::network_adapter_t& adapter) {
                            return adapter.name == network.name;
                        });
                    if (it3 != _deployment->instances().at(instanceId).config().networkAdapters.end())
                    {
                        *it3 = network;
                    }
                    else
                    {
                        _deployment->instances().at(instanceId).config().networkAdapters.emplace_back(network);
                    }
                    _app_db.insert_instance(instance);
                    _app_db.persist();
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
            _deployment->disconnect_network(instanceId, docker_network);
            _deployment->delete_network(docker_network);
            const auto it =
                std::find_if(instance.networks.cbegin(), instance.networks.cend(), [&](const std::string& str) {
                    return str == docker_network;
                });
            if (it != instance.networks.cend())
            {
                const auto pos = std::distance(instance.networks.cbegin(), it);
                instance.networks.erase(instance.networks.cbegin() + pos);
                instance.ips.erase(instance.ips.cbegin() + pos);
                _app_db.insert_instance(instance);
                _app_db.persist();
            }
            const auto it2 = std::find_if(
                _deployment->instances().at(instanceId).config().networks.cbegin(),
                _deployment->instances().at(instanceId).config().networks.cend(),
                [&](const instance_config_t::network_t network) { return network.network == docker_network; });
            if (it2 != _deployment->instances().at(instanceId).config().networks.cend())
            {
                _deployment->instances().at(instanceId).config().networks.erase(it2);
            }
            const auto it3 = std::find_if(
                _deployment->instances().at(instanceId).config().networkAdapters.cbegin(),
                _deployment->instances().at(instanceId).config().networkAdapters.cend(),
                [&](const instance_config_t::network_adapter_t& adapter) { return adapter.name == network.name; });
            if (it3 != _deployment->instances().at(instanceId).config().networkAdapters.cend())
            {
                _deployment->instances().at(instanceId).config().networkAdapters.erase(it3);
            }
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
