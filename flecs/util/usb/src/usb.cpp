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

#include "flecs/util/usb/usb.h"

#include <tuple>

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"

namespace flecs {
namespace usb {

auto operator<=>(const device_t& lhs, const device_t& rhs) //
    -> std::strong_ordering
{
    return std::tie(lhs.vid, lhs.pid, lhs.port) <=> std::tie(rhs.vid, rhs.pid, rhs.port);
}

auto operator==(const device_t& lhs, const device_t& rhs) //
    -> bool
{
    return lhs <=> rhs == std::strong_ordering::equal;
}

auto operator!=(const device_t& lhs, const device_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

auto to_json(json_t& json, const device_t& device) //
    -> void
{
    json = json_t(
        {{"device", device.device},
         {"pid", device.pid},
         {"port", device.port},
         {"vendor", device.vendor},
         {"vid", device.vid}});
}

auto from_json(const json_t& json, device_t& device) //
    -> void
{
    json.at("device").get_to(device.device);
    json.at("pid").get_to(device.pid);
    json.at("port").get_to(device.port);
    json.at("vendor").get_to(device.vendor);
    json.at("vid").get_to(device.vid);
}

auto get_devices() //
    -> std::set<device_t>
{
    auto devices = std::set<device_t>{};

    for (auto device : read_usb_devices()) {
        devices.insert(device_t{
            device.vid,
            device.pid,
            device.port.c_str(),
            device.device.c_str(),
            device.vendor.c_str()});
    }
    return devices;
}

} // namespace usb
} // namespace flecs
