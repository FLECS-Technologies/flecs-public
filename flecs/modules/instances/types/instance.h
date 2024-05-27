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

#include <memory>
#include <set>
#include <string>

#include "flecs/common/app/manifest/variable/variable.h"
#include "flecs/common/app/manifest/port_range/port_range.h"
#include "flecs/util/json/json.h"
#include "flecs/util/usb/usb.h"
#include "instance_config.h"
#include "instance_id.h"
#include "instance_status.h"

namespace flecs {
namespace apps {
class app_t;
} // namespace apps

namespace instances {

class instance_t
{
public:
    using envs_t = std::set<mapped_env_var_t>;
    using ports_t = std::vector<mapped_port_range_t>;
    struct network_t
    {
        std::string network_name;
        std::string mac_address;
        std::string ip_address;
    };

    instance_t();

    instance_t(std::shared_ptr<const apps::app_t> app, std::string instance_name);

    instance_t(instances::id_t id, std::shared_ptr<const apps::app_t> app, std::string instance_name);

    auto id() const noexcept //
        -> const instances::id_t&;
    auto app() const noexcept //
        -> std::shared_ptr<const apps::app_t>;
    auto app_name() const noexcept //
        -> std::string_view;
    auto app_version() const noexcept //
        -> std::string_view;
    auto has_app() const noexcept //
        -> bool;
    auto instance_name() const noexcept //
        -> const std::string&;
    auto status() const noexcept //
        -> instances::status_e;
    auto desired() const noexcept //
        -> instances::status_e;
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
    auto environment() const noexcept //
        -> envs_t;
    auto clear_environment() //
        -> void;
    auto set_environment(envs_t env) //
        -> void;
    auto ports() const noexcept //
        -> ports_t;
    auto clear_ports() //
        -> void;
    auto set_ports(ports_t ports) //
        -> void;
    auto regenerate_id() //
        -> void;
    auto app(std::shared_ptr<const apps::app_t> app) //
        -> void;
    auto instance_name(std::string instance_name) //
        -> void;
    auto status(instances::status_e instance_status) //
        -> void;
    auto desired(instances::status_e instance_status) //
        -> void;

private:
    friend auto to_json(json_t& json, const network_t& network) //
        -> void;
    friend auto from_json(const json_t& json, network_t& network) //
        -> void;

    friend auto to_json(json_t& json, const instance_t& instance) //
        -> void;
    friend auto from_json_v2(const json_t& json, instance_t& instance) //
        -> void;
    friend auto from_json_v1(const json_t& json, instance_t& instance) //
        -> void;
    friend auto from_json(const json_t& json, instance_t& instance) //
        -> void;

    instances::id_t _id;
    std::weak_ptr<const apps::app_t> _app;
    // std::weak_ptr<deployment_t> _deployment;
    std::string _app_name;
    std::string _app_version;
    std::string _instance_name;
    instances::status_e _status;
    instances::status_e _desired;
    std::vector<network_t> _networks;
    std::vector<unsigned> _startup_options;
    std::set<usb::device_t> _usb_devices;
    envs_t _env;
    ports_t _ports;
};

auto operator==(const instance_t& lhs, const instance_t& rhs) //
    -> bool;

} // namespace instances
} // namespace flecs
