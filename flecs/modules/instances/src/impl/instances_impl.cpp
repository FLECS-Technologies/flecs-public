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

#include "flecs/modules/instances/impl/instances_impl.h"

#include "flecs/api/api.h"
#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/deployments/types/deployment_docker.h"
#ifdef FLECS_MOCK_MODULES
#include "flecs/modules/apps/__mocks__/apps.h"
#include "flecs/modules/jobs/__mocks__/jobs.h"
#else // FLECS_MOCK_MODULES
#include "flecs/modules/apps/apps.h"
#include "flecs/modules/jobs/jobs.h"
#endif // FLECS_MOCK_MODULES
#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/util/datetime/datetime.h"
#include "flecs/util/network/network.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace module {
namespace impl {

namespace {
auto build_network_adapters_json(std::shared_ptr<instances::instance_t> instance)
{
    const auto adapters = get_network_adapters();
    auto adapters_json = json_t::array();
    for (decltype(auto) adapter : adapters) {
        if ((adapter.second.net_type == NetType::Wired) || (adapter.second.net_type == NetType::Wireless)) {
            auto adapter_json = json_t{};
            adapter_json["name"] = adapter.first;
            adapter_json["active"] = false;
            adapter_json["connected"] = !adapter.second.ipv4addresses.empty();
            auto network = std::string{"flecs-ipvlan_l2-"} + adapter.first;
            auto it = std::find_if(
                instance->networks().cbegin(),
                instance->networks().cend(),
                [&](const instances::instance_t::network_t& n) { return n.network_name == network; });
            if (it != instance->networks().cend()) {
                adapter_json["active"] = true;
                adapter_json["ipAddress"] = it->ip_address;
                if (adapter.second.ipv4addresses.empty()) {
                    adapter_json["subnetMask"] = "0.0.0.0";
                    adapter_json["gateway"] = "0.0.0.0";
                } else {
                    adapter_json["subnetMask"] = adapter.second.ipv4addresses.begin()->subnet_mask;
                    adapter_json["gateway"] = adapter.second.gateway;
                }
            }
            adapters_json.push_back(adapter_json);
        }
    }
    const auto net_prefix = std::string_view{"flecs-ipvlan_l2-"};
    for (decltype(auto) network : instance->networks()) {
        if (network.network_name.starts_with(net_prefix)) {
            const auto adapter = network.network_name.substr(net_prefix.size());
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

auto build_usb_devices_json(std::shared_ptr<instances::instance_t> instance)
{
    auto ret = json_t::array();
    const auto usb_devices = usb::get_devices();

    // insert connected usb device
    for (decltype(auto) usb_device : usb_devices) {
        auto device_json = json_t(usb_device);
        device_json["active"] = static_cast<bool>(instance->usb_devices().count(usb_device));
        device_json["connected"] = true;
        ret.push_back(std::move(device_json));
    }

    // insert configured, but disconnected usb devices
    for (decltype(auto) usb_device : instance->usb_devices()) {
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

instances_t::instances_t(flecs::module::instances_t* parent)
    : _parent{parent}
    , _deployment{new deployments::docker_t{}}
    , _apps_api{}
    , _jobs_api{}
{}

instances_t::~instances_t()
{}

auto instances_t::migrate_macvlan_to_ipvlan() //
    -> void
{
    // find all existing macvlan networks
    auto macvlan_networks = _deployment->networks();
    std::erase_if(macvlan_networks, [](decltype(macvlan_networks)::const_reference n) {
        return n.type != network_type_e::MACVLAN;
    });

    for (const auto& network : macvlan_networks) {
        const auto old_name = network.name;
        const auto new_name = std::string{old_name}.replace(network.name.find("macvlan"), 7, "ipvlan_l2");
        std::cout << "Migrating network " << old_name << " to " << new_name << "\n";
        // stop all instances connected to the macvlan network
        for (const auto& id : _parent->instance_ids()) {
            auto instance = _parent->query(id);
            auto it = std::find_if(
                instance->networks().begin(),
                instance->networks().end(),
                [&old_name](std::remove_cvref_t<decltype(instance->networks())>::const_reference n) {
                    return n.network_name == old_name;
                });
            if (it != instance->networks().end()) {
                _parent->stop_once(id);
                it->network_name = new_name;
            }
        }

        // delete macvlan network
        auto [res, message] = _deployment->delete_network(old_name);
        if (res != 0) {
            std::cerr << "Could not delete network " << old_name << ": " << message;
            continue;
        }

        // recreate network as ipvlan_l2
        std::tie(res, message) = _deployment->create_network(
            network_type_e::IPVLAN_L2,
            new_name,
            network.cidr_subnet,
            network.gateway,
            network.parent);
        if (res != 0) {
            std::cerr << "Could not create ipvlan network " << new_name << ": " << message;
            continue;
        }
    }
}

auto instances_t::do_load(const fs::path& base_path) //
    -> result_t
{
    _apps_api = std::dynamic_pointer_cast<module::apps_t>(api::query_module("apps"));
    _jobs_api = std::dynamic_pointer_cast<module::jobs_t>(api::query_module("jobs"));
    return _deployment->load(base_path);
}

auto instances_t::do_module_init() //
    -> void
{
    auto hosts_thread = std::thread([] {
        pthread_setname_np(pthread_self(), "flecs-update-hosts");
        auto hosts_process = process_t{};
        hosts_process.spawnp("sh", "-c", "/usr/local/bin/flecs-update-hosts.sh");
        hosts_process.wait(false, false);
    });
    hosts_thread.detach();

    migrate_macvlan_to_ipvlan();
}

auto instances_t::do_module_start() //
    -> void
{
    for (const auto& instance_id : _parent->instance_ids()) {
        if (auto instance = _parent->query(instance_id)) {
            instance->copy_missing_config_from_app_manifest();
            if (instance->desired() == instances::status_e::Running) {
                _parent->start_once(instance_id);
            }
        }
    }
}

auto instances_t::do_module_stop() //
    -> void
{
    for (const auto& instance_id : _parent->instance_ids()) {
        _parent->stop_once(instance_id);
    }
}

auto instances_t::do_instance_ids(const apps::key_t& app_key) const //
    -> std::vector<instances::id_t>
{
    return _deployment->instance_ids(app_key);
}

auto instances_t::do_query(instances::id_t instance_id) const //
    -> std::shared_ptr<instances::instance_t>
{
    return _deployment->query_instance(std::move(instance_id));
}

auto instances_t::do_is_running(std::shared_ptr<instances::instance_t> instance) const //
    -> bool
{
    return _deployment->is_instance_running(std::move(instance));
}

auto instances_t::queue_create(apps::key_t app_key, std::string instance_name, bool running) //
    -> jobs::id_t
{
    auto desc = "Creating new instance of " + to_string(app_key);

    auto job = jobs::job_t{std::bind(
        &instances_t::do_create,
        this,
        std::move(app_key),
        std::move(instance_name),
        std::move(running),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto instances_t::do_create_sync(apps::key_t app_key, std::string instance_name, bool running) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_create(std::move(app_key), std::move(instance_name), running, _);
}

auto instances_t::do_create(
    apps::key_t app_key, std::string instance_name, bool running, jobs::progress_t& progress) //
    -> result_t
{
    // Step 1: Ensure app is actually installed
    auto app = _apps_api->query(app_key);
    if (!app || (app->status() != apps::status_e::Installed)) {
        return {-1, "Could not create instance of " + to_string(app_key) + ": not installed"};
    }

    // Step 2: Load app manifest
    const auto manifest = app->manifest();
    if (!manifest || !manifest->is_valid()) {
        return {-1, "Could not create instance of " + to_string(app_key) + ": manifest error"};
    }

    // Step 3: Ensure there is only one instance of single-instance apps
    if (!manifest->multi_instance()) {
        const auto instance_ids = _deployment->instance_ids(app->key());
        if (!instance_ids.empty()) {
            auto instance = _deployment->query_instance(instance_ids.front());
            instance->app(std::move(app));

            return {0, instance->id().hex()};
        }
    }

    // Step 4: Forward to deployment
    const auto [res, instance_id] = _deployment->create_instance(std::move(app), instance_name);

    // Final step: Persist creation into db
    _deployment->save();

    if (res != 0) {
        return {-1, "Could not create instance of " + to_string(app_key)};
    }

    auto desc = progress.desc();
    desc += " -> " + instance_id;
    progress.desc(std::move(desc));

    if (running) {
        _parent->start(instances::id_t{instance_id});
    }

    return {0, instance_id};
}

auto instances_t::queue_start(instances::id_t instance_id, bool once) //
    -> jobs::id_t
{
    auto desc = "Starting instance " + instance_id.hex();

    auto job = jobs::job_t{std::bind(
        &instances_t::do_start,
        this,
        std::move(instance_id),
        std::move(once),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto instances_t::do_start_sync(instances::id_t instance_id, bool once) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_start(std::move(instance_id), std::move(once), _);
}

auto instances_t::do_start(instances::id_t instance_id, bool once, jobs::progress_t& progress) //
    -> result_t
{
    auto instance = _deployment->query_instance(instance_id);
    // Step 1: Verify instance does actually exist and is fully created
    if (!_deployment->is_instance_runnable(instance)) {
        return {-1, instance ? "Instance not fully created" : "Instance does not exist"};
    }

    auto desc = progress.desc();
    desc += " (" + to_string(instance->app()->key()) + ")";
    progress.desc(std::move(desc));
    // Step 3: Return if instance is already running
    if (_deployment->is_instance_running(instance)) {
        return {0, "Instance already running"};
    }

    // Step 3: Persist desired status into deployment
    if (!once) {
        instance->desired(instances::status_e::Running);
    }

    // Step 4: Forward to deployment
    const auto [res, additional_info] = _deployment->start_instance(instance);

    // Final step: Persist instance status into deployment
    _deployment->save();

    return {res, additional_info};
}

auto instances_t::queue_stop(instances::id_t instance_id, bool once) //
    -> jobs::id_t
{
    auto desc = "Stopping instance " + instance_id.hex();

    auto job = jobs::job_t{std::bind(
        &instances_t::do_stop,
        this,
        std::move(instance_id),
        std::move(once),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto instances_t::do_stop_sync(instances::id_t instance_id, bool once) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_stop(std::move(instance_id), std::move(once), _);
}

auto instances_t::do_stop(instances::id_t instance_id, bool once, jobs::progress_t& progress) //
    -> result_t
{
    // get instance details from database
    auto instance = _deployment->query_instance(instance_id);

    if (!instance) {
        return {-1, "Instance does not exist"};
    }

    auto desc = progress.desc();
    desc += " (" + to_string(instance->app()->key()) + ")";
    progress.desc(std::move(desc));

    // Step 3: Return if instance is not running
    if (!_deployment->is_instance_running(instance)) {
        return {0, "Instance not running"};
    }

    // Step 4: Persist desired status into db
    if (!once) {
        instance->desired(instances::status_e::Stopped);
    }

    // Step 5: Forward to deployment
    const auto [res, additional_info] = _deployment->stop_instance(instance);

    // Final step: Persist instance status into deployment
    _deployment->save();

    return {res, additional_info};
}

auto instances_t::queue_remove(instances::id_t instance_id) //
    -> jobs::id_t
{
    auto desc = "Removing instance " + instance_id.hex();

    auto job =
        jobs::job_t{std::bind(&instances_t::do_remove, this, std::move(instance_id), std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto instances_t::do_remove_sync(instances::id_t instance_id) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_remove(std::move(instance_id), _);
}

auto instances_t::do_remove(instances::id_t instance_id, jobs::progress_t& progress) //
    -> result_t
{
    progress.num_steps(3);

    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {-1, "Instance does not exist"};
    }

    auto desc = progress.desc();
    desc += " (" + to_string(instance->app()->key()) + ")";
    progress.desc(std::move(desc));

    // Step 2: Attempt to stop instance
    progress.next_step("Stopping instance");
    _deployment->stop_instance(instance);

    // Step 3: Remove volumes of instance
    progress.next_step("Removing volumes");
    _deployment->delete_volumes(instance);

    /// @todo: move functionality to deployment
    progress.next_step("Removing instance");

    auto [res, message] = _deployment->delete_instance(instance);
    _deployment->save();

    return {res, message};
}

auto instances_t::do_get_config(instances::id_t instance_id) const //
    -> crow::response
{
    auto response = json_t();

    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }

    response["networkAdapters"] = build_network_adapters_json(instance);
    response["devices"]["usb"] = build_usb_devices_json(instance);

    return {crow::status::OK, "json", response.dump()};
}

auto instances_t::do_post_config(instances::id_t instance_id, const instances::config_t& config) //
    -> crow::response
{
    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }

    auto response = json_t();
    response["networkAdapters"] = build_network_adapters_json(instance);

    const auto adapters = get_network_adapters();

    for (const auto& network : config.networkAdapters) {
        const auto docker_network = std::string{"flecs-ipvlan_l2-"} + network.name;
        if (network.active) {
            // ensure network adapter exists
            const auto netif = adapters.find(network.name);
            if (netif == adapters.cend()) {
                continue;
            }
            if (netif->second.ipv4addresses.empty()) {
                response["additionalInfo"] = "Network adapter " + netif->first + " not ready";
                continue;
            }

            // create ipvlan_l2 network, if not exists
            const auto cidr_subnet = ipv4_to_network(
                std::string{netif->second.ipv4addresses[0].addr},
                std::string{netif->second.ipv4addresses[0].subnet_mask});

            // process instance configuration
            if (network.ipAddress.empty()) {
                // suggest suitable IP address
                for (auto& adapter_json : response["networkAdapters"]) {
                    if (adapter_json["name"] == netif->first) {
                        adapter_json["active"] = true;
                        adapter_json["ipAddress"] = _deployment->generate_instance_ip(
                            cidr_subnet,
                            std::string{netif->second.gateway});
                        adapter_json["subnetMask"] = netif->second.ipv4addresses[0].subnet_mask;
                        adapter_json["gateway"] = netif->second.gateway;
                        break;
                    }
                }
            } else {
                // apply settings
                // @todo verify validity of IP address
                _deployment->create_network(
                    network_type_e::IPVLAN_L2,
                    docker_network,
                    cidr_subnet,
                    std::string{netif->second.gateway},
                    netif->first);

                _deployment->disconnect_network(instance, docker_network);

                const auto [res, additional_info] =
                    _deployment->connect_network(instance, docker_network, network.ipAddress);

                if (res == 0) {
                    auto it = std::find_if(
                        instance->networks().begin(),
                        instance->networks().end(),
                        [&](const instances::instance_t::network_t& n) {
                            return n.network_name == docker_network;
                        });
                    if (it != instance->networks().end()) {
                        it->ip_address = network.ipAddress;
                    } else {
                        instance->networks().emplace_back(instances::instance_t::network_t{
                            .network_name = docker_network,
                            .mac_address = {},
                            .ip_address = network.ipAddress});
                    }
                    _deployment->save();
                    for (auto& adapter_json : response["networkAdapters"]) {
                        if (adapter_json.contains("name") && (adapter_json["name"] == netif->first)) {
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
            _deployment->disconnect_network(instance, docker_network);
            _deployment->delete_network(docker_network);

            instance->networks().erase(
                std::remove_if(
                    instance->networks().begin(),
                    instance->networks().end(),
                    [&](const instances::instance_t::network_t& net) {
                        return net.network_name == docker_network;
                    }),
                instance->networks().end());

            for (auto& adapter_json : response["networkAdapters"]) {
                if (adapter_json.contains("name") && (adapter_json["name"] == network.name)) {
                    adapter_json["active"] = false;
                }
            }
        }
    }

    for (const auto& usb_device : config.usb_devices) {
        if (usb_device.active) {
            instance->usb_devices().emplace(usb_device);
        } else {
            instance->usb_devices().erase(usb_device);
        }
    }
    response["devices"]["usb"] = build_usb_devices_json(instance);

    return {crow::status::OK, "json", response.dump()};
}

auto instances_t::do_details(instances::id_t instance_id) const //
    -> crow::response
{
    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }

    auto response = json_t();
    // Step 2: Obtain corresponding app and manifest
    auto app = instance->app();
    if (!app) {
        response["additionalInfo"] = "Instance not connected to an App";
        return {crow::status::INTERNAL_SERVER_ERROR, "json", response.dump()};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        response["additionalInfo"] = "App not connected to a Manifest";
        return {crow::status::INTERNAL_SERVER_ERROR, "json", response.dump()};
    }

    // Implicitly use to_json function for instance_t for as much as possible
    response = *instance;

    response["ipAddress"] = instance->networks().empty() ? "" : instance->networks()[0].ip_address;

    // Fill missing content with information from manifest
    response["configFiles"] = json_t::array();
    for (const auto& config_file : manifest->conffiles()) {
        auto json = json_t{};
        json["host"] = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/" + config_file.local();
        json["container"] = config_file.container();
        response["configFiles"].push_back(json);
    }
    response["hostname"] =
        manifest->hostname().empty() ? ("flecs-" + instance->id().hex()) : manifest->hostname();
    response["ports"] = json_t::array();
    for (const auto& port : manifest->ports()) {
        auto json_port = json_t{};
        json_port["host"] = to_string(port.host_port_range());
        json_port["container"] = to_string(port.container_port_range());
        response["ports"].push_back(json_port);
    }
    response["volumes"] = json_t::array();
    for (const auto& volume : manifest->volumes()) {
        if (volume.type() == volume_t::VOLUME) {
            auto json_volume = json_t{};
            json_volume["name"] = volume.host();
            json_volume["path"] = volume.container();
            response["volumes"].push_back(json_volume);
        }
    }
    auto editors = json_t::array_t{};
    for (const auto& [_, editor] : manifest->editors()) {
        auto url = "/v2/instances/" + instance_id.hex() + "/editor/" + std::to_string(editor.port());
        editors.push_back({{"name", editor.name()}, {"url", url}});
    }
    response["editors"] = editors;

    return {crow::status::OK, "json", response.dump()};
}

auto instances_t::do_logs(instances::id_t instance_id) const //
    -> crow::response
{
    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }

    auto response = json_t{};

    // Step 2: Obtain log from Docker
    auto docker_process = process_t{};
    docker_process.spawnp("docker", "logs", "flecs-" + instance->id().hex());
    docker_process.wait(false, false);

    // Step 3: Build response
    if (docker_process.exit_code() != 0) {
        response["additionalInfo"] = "Could not get logs for instance " + instance->id().hex();
        return {crow::status::INTERNAL_SERVER_ERROR, "json", response.dump()};
    }

    response["stdout"] = docker_process.stdout();
    response["stderr"] = docker_process.stderr();

    return {crow::status::OK, "json", response.dump()};
}

auto instances_t::do_get_env(instances::id_t instance_id) const //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    json_t response =
        instance->environment().has_value() ? instance->environment().value() : std::set<mapped_env_var_t>{};
    return crow::response{crow::status::OK, "json", response.dump()};
}

auto instances_t::do_put_env(instances::id_t instance_id, std::vector<mapped_env_var_t> env_vars) //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    auto env = std::set<mapped_env_var_t>{};
    for (auto& env_var : env_vars) {
        auto result = env.insert(std::move(env_var));

        if (!result.second) {
            auto additionalInfo = json_t{};
            additionalInfo["additionalInfo"] = "Duplicate variable in environment: " + result.first->var();
            return crow::response{crow::status::BAD_REQUEST, "json", additionalInfo.dump()};
        }
    }
    instance->set_environment(std::move(env));
    _deployment->save();
    return crow::response{crow::status::OK};
}

auto instances_t::do_delete_env(instances::id_t instance_id) //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    instance->clear_environment();
    _deployment->save();
    return crow::response{crow::status::OK};
}

auto instances_t::do_get_ports(instances::id_t instance_id) const //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    json_t response =
        instance->ports().has_value() ? instance->ports().value() : std::vector<mapped_port_range_t>{};
    return crow::response{crow::status::OK, "json", response.dump()};
}

auto instances_t::do_put_ports(instances::id_t instance_id, std::vector<mapped_port_range_t> ports) //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    instance->set_ports(std::move(ports));
    _deployment->save();
    return crow::response{crow::status::OK};
}

auto instances_t::do_delete_ports(instances::id_t instance_id) //
    -> crow::response
{
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }
    instance->clear_ports();
    _deployment->save();
    return crow::response{crow::status::OK};
}

auto instances_t::queue_update(instances::id_t instance_id, std::string to) //
    -> jobs::id_t
{
    auto desc = "Updating instance " + instance_id.hex() + " to " + to;

    auto job = jobs::job_t{std::bind(
        &instances_t::do_update,
        this,
        std::move(instance_id),
        std::move(to),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}

auto instances_t::do_update_sync(instances::id_t instance_id, std::string to) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_update(std::move(instance_id), std::move(to), _);
}

auto instances_t::do_update(instances::id_t instance_id, std::string to, jobs::progress_t& /*progress*/) //
    -> result_t
{
    // Step 1: Verify instance does actually exist, is fully created and valid
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {-1, "Instance does not exist"};
    }
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an App"};
    }

    auto to_app_key = apps::key_t{app->key().name().data(), to};
    auto to_app = _apps_api->query(to_app_key);
    // Step 3: Make sure to-app is installed
    if (!to_app) {
        return {-1, "Updated App is not installed"};
    }

    // Step 4: Stop running instance
    auto [res, message] = _parent->stop_once(instance->id());
    if (res != 0) {
        return {-1, "Could not stop instance"};
    }

    // Step 5: Create backup of existing instance (volumes + config)
    using std::operator""s;
    const auto backup_path_base = fs::path{"/var/lib/flecs/backup/"} / instance->id().hex();
    const auto backup_path = backup_path_base / app->key().version().data() / unix_time(precision_e::seconds);
    std::tie(res, message) = _parent->export_to(instance->id(), backup_path);
    if (res != 0) {
        return {-1, "Could not backup instance"};
    }

    // Step 6: Restore previous backup on downgrade, if possible
    const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
    if (app->key().version() > to) {
        auto latest_path = fs::path{};
        auto ec = std::error_code{};
        auto backup_dir = fs::directory_iterator{backup_path_base / to, ec};
        for (; backup_dir != fs::directory_iterator{}; ++backup_dir) {
            if (backup_dir->path().filename() > latest_path.filename()) {
                latest_path = backup_dir->path();
            }
        }

        if (!latest_path.empty()) {
            _deployment->import_instance(instance, latest_path);
        }
    }

    // Step 7: Update instance structure
    instance->app(to_app);

    // Step 8: Persist updated instance into deployment
    _deployment->save();

    // Final step: Start instance
    if (instance->desired() == instances::status_e::Running) {
        std::tie(res, message) = _parent->start_once(instance->id());
        if (res != 0) {
            return {-1, "Could not start instance"};
        }
    }

    return {0, {}};
}

auto instances_t::queue_export_to(instances::id_t instance_id, fs::path base_path) //
    -> jobs::id_t
{
    auto desc = "Exporting instance " + instance_id.hex() + " to " + base_path.string();

    auto job = jobs::job_t{std::bind(
        &instances_t::do_export_to,
        this,
        std::move(instance_id),
        std::move(base_path),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}
auto instances_t::do_export_to_sync(instances::id_t instance_id, fs::path base_path) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_export_to(std::move(instance_id), std::move(base_path), _);
}
auto instances_t::do_export_to(
    instances::id_t instance_id, fs::path base_path, jobs::progress_t& /*progress*/) //
    -> result_t
{
    // Step 1: Verify instance does actually exist, is fully created and valid
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {-1, "Instance does not exist"};
    }

    const auto [res, additional_info] = _deployment->export_instance(instance, std::move(base_path));

    return {res, additional_info};
}

auto instances_t::queue_import_from(instances::instance_t instance, fs::path base_path) //
    -> jobs::id_t
{
    auto desc = "Importing instance " + instance.id().hex() + " from " + base_path.string();

    auto job = jobs::job_t{std::bind(
        &instances_t::do_import_from,
        this,
        std::move(instance),
        std::move(base_path),
        std::placeholders::_1)};

    return _jobs_api->append(std::move(job), std::move(desc));
}
auto instances_t::do_import_from_sync(instances::instance_t instance, fs::path base_path) //
    -> result_t
{
    auto _ = jobs::progress_t{};
    return do_import_from(std::move(instance), std::move(base_path), _);
}
auto instances_t::do_import_from(
    instances::instance_t instance, fs::path base_path, jobs::progress_t& /*progress*/) //
    -> result_t
{
    auto app = _apps_api->query(apps::key_t{instance.app_name().data(), instance.app_version().data()});
    if (!app) {
        return {-1, "App is not installed"};
    }
    instance.app(std::move(app));

    auto p = _deployment->query_instance(instance.id());
    if (!p) {
        p = _deployment->insert_instance(std::move(instance));
    } else {
        *p = std::move(instance);
    }
    auto [res, message] = _deployment->import_instance(p, base_path);

    return {res, message};
}

} // namespace impl
} // namespace module
} // namespace flecs
