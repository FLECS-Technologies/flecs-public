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

#pragma once

#include <cstdint>
#include <set>
#include <string>

#include "util/json/json.h"

namespace FLECS {
namespace usb {

struct device_t
{
    std::uint16_t pid;
    std::uint16_t vid;
    std::string device;
    std::string port;
    std::string vendor;
};

bool operator<(const device_t& lhs, const device_t& rhs);
bool operator<=(const device_t& lhs, const device_t& rhs);
bool operator>(const device_t& lhs, const device_t& rhs);
bool operator>=(const device_t& lhs, const device_t& rhs);
bool operator==(const device_t& lhs, const device_t& rhs);
bool operator!=(const device_t& lhs, const device_t& rhs);

auto to_json(json_t& json, const device_t& device) //
    -> void;

auto from_json(const json_t& json, device_t& device) //
    -> void;

auto get_devices() //
    -> std::set<device_t>;

} // namespace usb
} // namespace FLECS
