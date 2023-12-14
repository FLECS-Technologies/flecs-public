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

#include "daemon/modules/instances/types/instance_config.h"

namespace flecs {
namespace instances {

auto to_json(json_t& json, const config_t::network_adapter_t& network_adapter) //
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

auto to_json(json_t& json, const config_t::usb_device_t& usb_device) //
    -> void
{
    json = json_t(static_cast<const usb::device_t&>(usb_device));
    json["active"] = usb_device.active;
}

auto to_json(json_t& json, const config_t& instance_config) //
    -> void
{
    json = json_t{
        {"networkAdapters", instance_config.networkAdapters},
        {"startupOptions", instance_config.startup_options},
    };
    json["devices"] = json_t::array();
    json["device"]["usb"] = instance_config.usb_devices;
}

auto from_json(const json_t& json, config_t::network_adapter_t& network_adapter) //
    -> void
{
    if (json.contains("name")) {
        json.at("name").get_to(network_adapter.name);
    }
    if (json.contains("ipAddress")) {
        json.at("ipAddress").get_to(network_adapter.ipAddress);
    }
    if (json.contains("subnetMask")) {
        json.at("subnetMask").get_to(network_adapter.subnetMask);
    }
    if (json.contains("gateway")) {
        json.at("gateway").get_to(network_adapter.gateway);
    }
    if (json.contains("active")) {
        json.at("active").get_to(network_adapter.active);
    }
}

auto from_json(const json_t& json, config_t::usb_device_t& usb_device) //
    -> void
{
    static_cast<usb::device_t&>(usb_device) = json.get<usb::device_t>();
    json.at("active").get_to(usb_device.active);
}

auto from_json(const json_t& json, config_t& instance_config) //
    -> void
{
    json.at("networkAdapters").get_to(instance_config.networkAdapters);
    json.at("startupOptions").get_to(instance_config.startup_options);
}

} // namespace instances
} // namespace flecs
