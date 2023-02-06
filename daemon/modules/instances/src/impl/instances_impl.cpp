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

#include "impl/instances_impl.h"

#include "api/api.h"
#include "common/app/app.h"
#include "common/app/manifest/manifest.h"
#include "common/deployment/deployment_docker.h"
#include "modules/apps/apps.h"
#include "modules/factory/factory.h"
#include "modules/jobs/jobs.h"
#include "modules/system/system.h"
#include "util/cxx20/string.h"
#include "util/network/network.h"
#include "util/process/process.h"

namespace FLECS {
namespace impl {

namespace {
auto build_network_adapters_json(std::shared_ptr<instance_t> instance)
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
                instance->networks().cbegin(),
                instance->networks().cend(),
                [&](const instance_t::network_t& n) { return n.network_name == network; });
            if (it != instance->networks().cend()) {
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
    for (decltype(auto) network : instance->networks()) {
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

auto build_usb_devices_json(std::shared_ptr<instance_t> instance)
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

module_instances_t::module_instances_t(FLECS::module_instances_t* parent)
    : _parent{parent}
    , _deployment{new deployment_docker_t{}}
    , _apps_api{}
    , _jobs_api{}
{}

module_instances_t::~module_instances_t()
{}

auto module_instances_t::do_init() //
    -> void
{
    _apps_api = std::dynamic_pointer_cast<FLECS::module_apps_t>(api::query_module("apps"));
    _jobs_api = std::dynamic_pointer_cast<FLECS::module_jobs_t>(api::query_module("jobs"));
}

auto module_instances_t::do_list(const app_key_t& app_key) const //
    -> crow::response
{
    auto response = json_t::array();

    for (const auto& instance_id : _deployment->instance_ids(app_key, deployment_t::MatchVersion)) {
        auto json = json_t::object();

        auto instance = _deployment->query_instance(instance_id);

        json["instanceId"] = instance->id().hex();
        if (auto app = instance->app()) {
            json["appKey"] = app->key();
            if (instance->status() == instance_status_e::Created) {
                json["status"] = to_string(
                    _deployment->is_instance_running(instance) ? instance_status_e::Running
                                                               : instance_status_e::Stopped);
            } else {
                json["status"] = to_string(instance->status());
            }
        } else {
            json["appKey"] = app_key_t{};
            json["status"] = instance_status_e::Orphaned;
        }
        json["desired"] = to_string(instance->desired());
        response.push_back(std::move(json));
    }

    return {crow::status::OK, "json", response.dump()};
}

auto module_instances_t::queue_create(app_key_t app_key, std::string instance_name) //
    -> crow::response
{
    auto job = job_t{};
    job.callable = std::bind(
        &module_instances_t::do_create,
        this,
        std::move(app_key),
        std::move(instance_name),
        std::placeholders::_1);

    auto job_id = _jobs_api->append(std::move(job));

    auto response = json_t::object();
    response["jobId"] = std::to_string(job_id);
    return {crow::status::ACCEPTED, "json", response.dump()};
}

auto module_instances_t::do_create(
    app_key_t app_key, std::string instance_name, job_progress_t& progress) //
    -> void
{
    // Step 1: Ensure app is actually installed
    auto app = _apps_api->query(app_key);
    if (!app || (app->status() != app_status_e::Installed)) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message =
            "Could not create instance of " + to_string(app_key) + ": not installed";
        return;
    }

    // Step 2: Load app manifest
    const auto manifest = app->manifest();
    if (!manifest || !manifest->is_valid()) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message =
            "Could not create instance of " + to_string(app_key) + ": manifest error";
        return;
    }

    // Step 3: Ensure there is only one instance of single-instance apps
    if (!manifest->multi_instance()) {
        const auto instance_ids = _deployment->instance_ids(app->key(), deployment_t::MatchVersion);
        if (!instance_ids.empty()) {
            auto instance = _deployment->query_instance(instance_ids.front());
            instance->app(std::move(app));

            auto lock = progress.lock();
            progress.result().code = 0;
            progress.result().message = instance->id().hex();
            return;
        }
    }

    // Step 4: Forward to deployment
    const auto [res, instance_id] = _deployment->create_instance(std::move(app), instance_name);

    // Final step: Persist creation into db
    _deployment->save();

    if (res != 0) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message = "Could not create instance of " + to_string(app_key);
        return;
    }

    auto lock = progress.lock();
    progress.result().code = 0;
    progress.result().message = instance_id;
}

auto module_instances_t::queue_start(instance_id_t instance_id) //
    -> crow::response
{
    auto job = job_t{};
    job.callable = std::bind(
        &module_instances_t::do_start,
        this,
        std::move(instance_id),
        std::placeholders::_1);

    auto job_id = _jobs_api->append(std::move(job));

    auto response = json_t::object();
    response["jobId"] = std::to_string(job_id);
    return {crow::status::ACCEPTED, "json", response.dump()};
}

auto module_instances_t::do_start(instance_id_t instance_id, job_progress_t& progress) //
    -> void
{
    auto instance = _deployment->query_instance(instance_id);
    // Step 1: Verify instance does actually exist and is fully created
    if (!_deployment->is_instance_runnable(instance)) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message =
            instance ? "Instance not fully created" : "Instance does not exist";
    }

    // Step 3: Return if instance is already running
    if (_deployment->is_instance_running(instance)) {
        auto lock = progress.lock();
        progress.result().message = "Instance already running";
    }

    // Step 3: Persist desired status into deployment
    /** @todo implement start_once without persistence */
    instance->desired(instance_status_e::Running);

    // Step 4: Forward to deployment
    const auto [res, additional_info] = _deployment->start_instance(instance);

    // Final step: Persist instance status into deployment
    _deployment->save();

    auto lock = progress.lock();
    progress.result().code = std::move(res);
    progress.result().message = std::move(additional_info);
}

auto module_instances_t::queue_stop(instance_id_t instance_id) //
    -> crow::response
{
    auto job = job_t{};
    job.callable = std::bind(
        &module_instances_t::do_stop,
        this,
        std::move(instance_id),
        std::placeholders::_1);

    auto job_id = _jobs_api->append(std::move(job));

    auto response = json_t::object();
    response["jobId"] = std::to_string(job_id);
    return {crow::status::ACCEPTED, "json", response.dump()};
}

auto module_instances_t::do_stop(instance_id_t instance_id, job_progress_t& progress) //
    -> void
{
    // get instance details from database
    auto instance = _deployment->query_instance(instance_id);

    if (!instance) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message = "Instance does not exist";
    }

    // Step 3: Return if instance is not running
    if (!_deployment->is_instance_running(instance)) {
        auto lock = progress.lock();
        progress.result().message = "Instance not running";
    }

    // Step 4: Persist desired status into db
    instance->desired(instance_status_e::Stopped);

    // Step 5: Forward to deployment
    const auto [res, additional_info] = _deployment->stop_instance(instance);

    // Final step: Persist instance status into deployment
    _deployment->save();

    auto lock = progress.lock();
    progress.result().code = std::move(res);
    progress.result().message = std::move(additional_info);
}

auto module_instances_t::queue_remove(instance_id_t instance_id) //
    -> crow::response
{
    auto job = job_t{};
    job.callable = std::bind(
        &module_instances_t::do_remove,
        this,
        std::move(instance_id),
        std::placeholders::_1);

    auto job_id = _jobs_api->append(std::move(job));

    auto response = json_t::object();
    response["jobId"] = std::to_string(job_id);
    return {crow::status::ACCEPTED, "json", response.dump()};
}

auto module_instances_t::do_remove(
    instance_id_t instance_id,
    job_progress_t& progress) //
    -> void
{
    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        auto lock = progress.lock();
        progress.result().code = -1;
        progress.result().message = "Instance does not exist";
    }

    // Step 2: Attempt to stop instance
    _deployment->stop_instance(instance);

    // Step 3: Remove volumes of instance
    _deployment->delete_volumes(instance);

    /// @todo: move functionality to deployment
    _deployment->delete_instance(instance);
    _deployment->save();
}

auto module_instances_t::do_get_config(instance_id_t instance_id) const //
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

    return {crow::status::OK, "json", {}};
}

auto module_instances_t::do_post_config(
    instance_id_t instance_id, const instance_config_t& config) //
    -> crow::response
{
    // Step 1: Verify instance does actually exist
    auto instance = _deployment->query_instance(instance_id);
    if (!instance) {
        return {crow::status::NOT_FOUND, {}};
    }

    auto response = json_t();
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

                _deployment->disconnect_network(instance, docker_network);

                const auto [res, additional_info] =
                    _deployment->connect_network(instance, docker_network, network.ipAddress);

                if (res == 0) {
                    auto it = std::find_if(
                        instance->networks().begin(),
                        instance->networks().end(),
                        [&](const instance_t::network_t& n) {
                            return n.network_name == docker_network;
                        });
                    if (it != instance->networks().end()) {
                        it->ip_address = network.ipAddress;
                    } else {
                        instance->networks().emplace_back(instance_t::network_t{
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
            _deployment->disconnect_network(instance, docker_network);
            _deployment->delete_network(docker_network);

            instance->networks().erase(
                std::remove_if(
                    instance->networks().begin(),
                    instance->networks().end(),
                    [&](const instance_t::network_t& net) {
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

    return {crow::status::OK, "json", {}};
}

auto module_instances_t::do_details(instance_id_t instance_id) const //
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

    // Build response
    response["instanceId"] = instance->id().hex();
    response["appKey"] = app->key();
    response["status"] = to_string(instance->status());
    response["desired"] = to_string(instance->desired());
    response["ipAddress"] = instance->networks().empty() ? "" : instance->networks()[0].ip_address;
    response["configFiles"] = json_t::array();
    for (const auto& config_file : manifest->conffiles()) {
        auto json = json_t{};
        json["host"] =
            "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/" + config_file.local();
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

    return {crow::status::OK, "json", response.dump()};
}

auto module_instances_t::do_logs(instance_id_t instance_id) const //
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

auto module_instances_t::queue_update(
    instance_id_t /*instance_id*/, std::string /*from*/, std::string /*to*/) //
    -> crow::response
{
    return {crow::status::ACCEPTED, "json", {}};
}

auto module_instances_t::do_update(
    instance_id_t /*instance_id*/,
    std::string /*from*/,
    std::string /*to*/,
    job_progress_t& /*progress*/) //
    -> void
{}

auto module_instances_t::queue_export(instance_id_t /*instance_id*/) //
    -> crow::response
{
    return {crow::status::ACCEPTED, "json", {}};
}

auto module_instances_t::do_export(instance_id_t /*instance_id*/, job_progress_t& /*progress*/) //
    -> void
{}

} // namespace impl
} // namespace FLECS
