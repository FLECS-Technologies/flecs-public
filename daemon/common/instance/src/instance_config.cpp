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

auto to_json(json_t& json, const instance_config_t::usb_device_t& usb_device) //
    -> void
{
    json = json_t({
        {"active", usb_device.active},
        {"pid", usb_device.pid},
        {"port", usb_device.port},
        {"vid", usb_device.vid},
    });
}

auto to_json(json_t& json, const instance_config_t& instance_config) //
    -> void
{
    json = json_t{
        {"networkAdapters", instance_config.networkAdapters},
        {"startupOptions", instance_config.startup_options},
    };
    json["devices"] = json_t::array();
    json["device"]["usb"] = instance_config.usbDevices;
}

auto from_json(const json_t& json, instance_config_t::network_adapter_t& network_adapter) //
    -> void
{
    if (json.contains("name"))
    {
        json.at("name").get_to(network_adapter.name);
    }
    if (json.contains("ipAddress"))
    {
        json.at("ipAddress").get_to(network_adapter.ipAddress);
    }
    if (json.contains("subnetMask"))
    {
        json.at("subnetMask").get_to(network_adapter.subnetMask);
    }
    if (json.contains("gateway"))
    {
        json.at("gateway").get_to(network_adapter.gateway);
    }
    if (json.contains("active"))
    {
        json.at("active").get_to(network_adapter.active);
    }
}

auto from_json(const json_t& json, instance_config_t::usb_device_t& usb_device) //
    -> void
{
    json.at("active").get_to(usb_device.active);
    json.at("pid").get_to(usb_device.pid);
    json.at("port").get_to(usb_device.port);
    json.at("vid").get_to(usb_device.vid);
}

auto from_json(const json_t& json, instance_config_t& instance_config) //
    -> void
{
    json.at("networkAdapters").get_to(instance_config.networkAdapters);
    json.at("startupOptions").get_to(instance_config.startup_options);
}

} // namespace FLECS
