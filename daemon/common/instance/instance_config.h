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

#ifndef E8E3AE12_7249_481B_B47C_5682C1BBADE2
#define E8E3AE12_7249_481B_B47C_5682C1BBADE2

#include <cstdint>
#include <string>
#include <unordered_map>
#include <vector>

#include "util/json/json.h"
#include "util/usb/usb.h"

namespace FLECS {

struct instance_config_t
{
    struct network_adapter_t
    {
        std::string name;
        std::string ipAddress;
        std::string subnetMask;
        std::string gateway;
        bool active;
    };
    std::vector<network_adapter_t> networkAdapters;

    struct usb_device_t : usb::device_t
    {
        bool active;
    };
    std::vector<usb_device_t> usb_devices;

    std::vector<unsigned> startup_options;
};

auto to_json(json_t& json, const instance_config_t::network_adapter_t& network_adapter) //
    -> void;
auto to_json(json_t& json, const instance_config_t::usb_device_t& usb_device) //
    -> void;
auto to_json(json_t& json, const instance_config_t& instance_config) //
    -> void;

auto from_json(const json_t& json, instance_config_t::network_adapter_t& network_adapter) //
    -> void;
auto from_json(const json_t& json, instance_config_t::usb_device_t& usb_device) //
    -> void;
auto from_json(const json_t& json, instance_config_t& instance_config) //
    -> void;

} // namespace FLECS

#endif // E8E3AE12_7249_481B_B47C_5682C1BBADE2
