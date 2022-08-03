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

#include "instance_config.h"

namespace FLECS {

auto to_json(json_t& json, const instance_config_t::network_adapter_t& network_adapter) //
    -> void
{
    json = json_t({
        {"name", network_adapter.name},
        {"ipAddress", network_adapter.ipAddress},
        {"subnetMask", network_adapter.subnetMask},
        {"gateway", network_adapter.gateway},
        {"active", network_adapter.active},
    });
}

auto to_json(json_t& json, const instance_config_t& instance_config) //
    -> void
{
    json = json_t{
        {"networkAdapters", instance_config.networkAdapters},
        {"startupOptions", instance_config.startup_options},
    };
}

auto from_json(const json_t& json, instance_config_t::network_adapter_t& network_adapter) //
    -> void
{
    json.at("name").get_to(network_adapter.name);
    json.at("ipAddress").get_to(network_adapter.ipAddress);
    json.at("subnetMask").get_to(network_adapter.subnetMask);
    json.at("gateway").get_to(network_adapter.gateway);
    json.at("active").get_to(network_adapter.active);
}

auto from_json(const json_t& json, instance_config_t& instance_config) //
    -> void
{
    json.at("networkAdapters").get_to(instance_config.networkAdapters);
    json.at("startupOptions").get_to(instance_config.startup_options);
}

} // namespace FLECS