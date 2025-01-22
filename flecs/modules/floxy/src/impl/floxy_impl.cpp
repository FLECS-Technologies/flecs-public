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

#include "flecs/modules/floxy/impl/floxy_impl.h"

#include <netinet/in.h>
#include <rust/cxx.h>
#include <sys/socket.h>

#include <fstream>
#include <string>

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"
#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/instances/instances.h"
#include "flecs/modules/instances/types/instance.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace module {
namespace impl {

auto floxy_t::do_load_instance_reverse_proxy_config(
    const std::string& ip_address,
    const std::string& app_name,
    const instances::id_t& instance_id,
    std::vector<std::uint16_t>& dest_ports) //
    -> result_t
{
    std::sort(dest_ports.begin(), dest_ports.end());
    auto ports = rust::Vec<uint16_t>{};
    for (auto port : dest_ports) {
        ports.push_back(port);
    }
    try {
        load_instance_reverse_proxy_config(app_name.c_str(), instance_id.get(), ip_address.c_str(), ports);
    }   catch (const rust::Error& e) {
        return {-1, e.what()};
    }
    return {0, {}};
}

auto floxy_t::do_redirect_editor_request(instances::id_t instance_id, std::uint16_t port) //
    -> crow::response
{
    const auto instances_api = dynamic_cast<module::instances_t*>(api::query_module("instances").get());
    if (!instances_api) {
        return {crow::status::INTERNAL_SERVER_ERROR, {}};
    }
    auto instance = instances_api->query(instance_id);
    if (!instance) {
        auto json = json_t{{"additionalInfo", "Instance not found"}};
        return {crow::status::NOT_FOUND, json.dump()};
    }
    const auto& editors = instance->app()->manifest()->editors();
    auto editor = editors.find(port);
    if (editor == editors.end()) {
        auto json = json_t{{"additionalInfo", "Unknown port"}};
        return {crow::status::NOT_FOUND, json.dump()};
    }
    if (editor->second.supports_reverse_proxy()) {
        auto response = json_t{{"additionalInfo", "Editor supports reverse proxy -> use floxy"}};
        return crow::response{crow::status::BAD_REQUEST, response.dump()};
    }
    if (!instances_api->is_running(instance)) {
        auto json = json_t{{"additionalInfo", "Instance is not running"}};
        return crow::response{crow::status::BAD_REQUEST, json.dump()};
    }
    auto response = crow::response{};
    auto& mapping = instance->editor_port_mapping();
    auto result = mapping.find(port);
    if (result != mapping.end()) {
        response.moved(":" + to_string(result->second));
    } else {
        std::optional<std::string> instance_ip;
        for (const auto& network : instance->networks()) {
            if (network.network_name == "flecs") {
                instance_ip = network.ip_address;
                break;
            }
        }
        if (!instance_ip.has_value()) {
            auto json = json_t{{"additionalInfo", "Instance not connected to network"}};
            return crow::response{crow::status::INTERNAL_SERVER_ERROR, json.dump()};
        }
        auto host_port = 0;
        try {
            host_port = create_instance_editor_redirect_to_free_port(
                instance->app_name().data(), instance_id.get(), instance_ip.value(), port);
        } catch (const rust::Error& e) {
            auto json = json_t{{"additionalInfo", e.what()}};
            return crow::response{crow::status::INTERNAL_SERVER_ERROR, json.dump()};
        }
        instance->set_editor_port_mapping(host_port, port);
        response.moved(":" + to_string(host_port));
    }
    return response;
}

auto floxy_t::do_delete_reverse_proxy_configs(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app_name = instance->app()->key().name();
    auto ports = rust::Vec<std::uint16_t>{};
    for (auto host_port : instance->editor_port_mapping() | std::views::values) {
        ports.push_back(host_port);
    }
    try {
        delete_reverse_proxy_configs(app_name.c_str(), instance->id().get(), ports);
    } catch (const rust::Error& e) {
        return {-1, e.what()};
    }
    return {0, {}};
}

auto floxy_t::do_delete_server_proxy_configs(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app_name = instance->app()->key().name();
    auto ports = rust::Vec<std::uint16_t>{};
    for (auto host_port : instance->editor_port_mapping() | std::views::values) {
        ports.push_back(host_port);
    }
    try {
        delete_server_proxy_configs(app_name.c_str(), instance->id().get(), ports);
    } catch (const rust::Error& e) {
        return {-1, e.what()};
    }
    return {0, {}};
}

} // namespace impl
} // namespace module
} // namespace flecs
