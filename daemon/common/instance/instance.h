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

#ifndef BFBA07F5_D230_427A_814B_DA012423C246
#define BFBA07F5_D230_427A_814B_DA012423C246

#include <memory>
#include <set>
#include <string>

#include "instance_config.h"
#include "instance_status.h"
#include "util/json/json.h"
#include "util/usb/usb.h"

namespace FLECS {

class app_t;

class instance_t
{
public:
    struct network_t
    {
        std::string network_name;
        std::string mac_address;
        std::string ip_address;
    };

    instance_t();

    instance_t(const app_t* app, std::string instance_name, instance_status_e status, instance_status_e desired);

    instance_t(
        std::string id,
        const app_t* app,
        std::string instance_name,
        instance_status_e status,
        instance_status_e desired);

    auto id() const noexcept //
        -> const std::string&;
    auto app() const noexcept //
        -> const app_t&;
    auto app_name() const noexcept //
        -> const std::string&;
    auto app_version() const noexcept //
        -> const std::string&;
    auto instance_name() const noexcept //
        -> const std::string&;
    auto status() const noexcept //
        -> instance_status_e;
    auto desired() const noexcept //
        -> instance_status_e;
    auto networks() const noexcept //
        -> const std::vector<network_t>&;
    auto networks() noexcept //
        -> std::vector<network_t>&;
    auto startup_options() const noexcept //
        -> const std::vector<unsigned>&;
    auto startup_options() noexcept //
        -> std::vector<unsigned>&;
    auto usb_devices() const noexcept //
        -> const std::set<usb::device_t>&;
    auto usb_devices() noexcept //
        -> std::set<usb::device_t>&;

    auto regenerate_id() //
        -> void;
    auto app(const app_t* app) //
        -> void;
    auto instance_name(std::string instance_name) //
        -> void;
    auto status(instance_status_e instance_status) //
        -> void;
    auto desired(instance_status_e instance_status) //
        -> void;

private:
    friend auto to_json(json_t& json, const network_t& network) //
        -> void;
    friend auto from_json(const json_t& json, network_t& network) //
        -> void;

    friend auto to_json(json_t& json, const instance_t& instance) //
        -> void;
    friend auto from_json(const json_t& json, instance_t& instance) //
        -> void;

    std::string _id;
    const app_t* _app;
    std::string _app_name;
    std::string _app_version;
    std::string _instance_name;
    instance_status_e _status;
    instance_status_e _desired;
    std::vector<network_t> _networks;
    std::vector<unsigned> _startup_options;
    std::set<usb::device_t> _usb_devices;
};

auto operator==(const instance_t& lhs, const instance_t& rhs) //
    -> bool;

} // namespace FLECS

#endif // BFBA07F5_D230_427A_814B_DA012423C246
