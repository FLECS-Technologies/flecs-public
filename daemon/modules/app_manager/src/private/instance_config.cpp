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
            auto network = std::string{"flecs-ipvlan-"} + adapter.first;
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

http_status_e module_app_manager_private_t::do_post_config_instance(const std::string& instanceId, json_t& response)
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instanceId;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({instanceId}))
    {
        response["additionalInfo"] = "Could not configure instance " + instanceId + ", which does not exist";
        return http_status_e::BadRequest;
    }

    const auto instance = _app_db.query_instance({instanceId}).value();
    response["networkAdapters"] = build_network_adapters_json(instance);

    return http_status_e::Ok;
}

http_status_e module_app_manager_private_t::do_put_config_instance(
    const std::string& instanceId, const instance_config_t& config, json_t& response)
{
    response["additionalInfo"] = std::string{};
    response["instanceId"] = instanceId;

    // Step 1: Verify instance does actually exist
    if (!_app_db.has_instance({instanceId}))
    {
        response["additionalInfo"] = "Could not configure instance " + instanceId + ", which does not exist";
        return http_status_e::BadRequest;
    }

    auto instance = _app_db.query_instance({instanceId}).value();
    response["networkAdapters"] = build_network_adapters_json(instance);

    const auto system_api = dynamic_cast<const module_system_t*>(api::query_module("system").get());
    const auto adapters = system_api->get_network_adapters();

    for (const auto& network : config.networkAdapters)
    {
        const auto docker_network = std::string{"flecs-ipvlan-"} + network.name;
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

            // create ipvlan network, if not exists
            const auto subnet =
                ipv4_to_network(netif->second.ipv4_addr[0].addr, netif->second.ipv4_addr[0].subnet_mask);
            auto docker_process = process_t{};
            docker_process.spawnp(
                "docker",
                "network",
                "create",
                "-d",
                "ipvlan",
                "--subnet",
                subnet,
                "--gateway",
                netif->second.gateway,
                "-o",
                std::string{"parent="} + netif->first,
                docker_network);
            docker_process.wait(false, false);

            // process instance configuration
            if (network.ipAddress.empty())
            {
                // suggest suitable IP address
                for (auto& adapter_json : response["networkAdapters"])
                {
                    if (adapter_json["name"] == netif->first)
                    {
                        adapter_json["active"] = true;
                        adapter_json["ipAddress"] = generate_ip(subnet);
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
                {
                    auto docker_process = process_t{};
                    docker_process
                        .spawnp("docker", "network", "disconnect", docker_network, std::string{"flecs-"} + instanceId);
                    docker_process.wait(false, false);
                }

                auto docker_process = process_t{};
                docker_process.spawnp(
                    "docker",
                    "network",
                    "connect",
                    "--ip",
                    network.ipAddress,
                    docker_network,
                    std::string{"flecs-"} + instanceId);
                docker_process.wait(false, false);
                if (docker_process.exit_code() == 0)
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
                    response["additionalInfo"] = "Could not connect to ipvlan network";
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
            auto docker_process = process_t{};
            docker_process
                .spawnp("docker", "network", "disconnect", docker_network, std::string{"flecs-"} + instanceId);
            docker_process.wait(false, false);
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
            for (auto& adapter_json : response["networkAdapters"])
            {
                if (adapter_json.contains("name") && (adapter_json["name"] == network.name))
                {
                    adapter_json["active"] = false;
                }
            }
        }
    }
    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
