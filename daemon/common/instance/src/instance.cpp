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

#include "daemon/common/instance/instance.h"

#include <cstdio>

#include "daemon/modules/apps/types/app.h"
#include "util/random/random.h"
#include "util/string/format.h"

namespace flecs {

instance_t::instance_t()
    : instance_t{instance_id_t{}, nullptr, std::string{}}
{}

instance_t::instance_t(std::shared_ptr<const apps::app_t> app, std::string instance_name)
    : instance_t{instance_id_t{}, app, std::move(instance_name)}
{}

instance_t::instance_t(instance_id_t id, std::shared_ptr<const apps::app_t> app, std::string instance_name)
    : _id{std::move(id)}
    , _app{app}
    , _app_name{app ? app->key().name() : ""}
    , _app_version{app ? app->key().version() : ""}
    , _instance_name{std::move(instance_name)}
    , _status{instance_status_e::Unknown}
    , _desired{instance_status_e::Unknown}
    , _networks{}
    , _startup_options{}
{}

auto instance_t::id() const noexcept //
    -> const instance_id_t&
{
    return _id;
}

auto instance_t::app() const noexcept //
    -> std::shared_ptr<const apps::app_t>
{
    return _app.lock();
}

auto instance_t::app_name() const noexcept //
    -> std::string_view
{
    const auto p = app();
    return p ? p->key().name() : _app_name;
}

auto instance_t::app_version() const noexcept //
    -> std::string_view
{
    const auto p = app();
    return p ? p->key().version() : _app_version;
}

auto instance_t::has_app() const noexcept //
    -> bool
{
    return !_app.expired();
}

auto instance_t::instance_name() const noexcept //
    -> const std::string&
{
    return _instance_name;
}

auto instance_t::status() const noexcept //
    -> instance_status_e
{
    return _status;
}

auto instance_t::desired() const noexcept //
    -> instance_status_e
{
    return _desired;
}

auto instance_t::networks() const noexcept //
    -> const std::vector<instance_t::network_t>&
{
    return _networks;
}

auto instance_t::networks() noexcept //
    -> std::vector<instance_t::network_t>&
{
    return _networks;
}

auto instance_t::startup_options() const noexcept //
    -> const std::vector<unsigned>&
{
    return _startup_options;
}

auto instance_t::startup_options() noexcept //
    -> std::vector<unsigned>&
{
    return _startup_options;
}

auto instance_t::usb_devices() const noexcept //
    -> const std::set<usb::device_t>&
{
    return _usb_devices;
}

auto instance_t::usb_devices() noexcept //
    -> std::set<usb::device_t>&
{
    return _usb_devices;
}

auto instance_t::regenerate_id() //
    -> void
{
    return _id.regenerate();
}

auto instance_t::app(std::shared_ptr<const apps::app_t> app) //
    -> void
{
    _app = app;
}

auto instance_t::instance_name(std::string instance_name) //
    -> void
{
    _instance_name = instance_name;
}

auto instance_t::status(instance_status_e status) //
    -> void
{
    _status = status;
}

auto instance_t::desired(instance_status_e desired) //
    -> void
{
    _desired = desired;
}

auto to_json(json_t& json, const instance_t::network_t& network) //
    -> void
{
    json = json_t{
        {"ipAddress", network.ip_address},
        {"macAddress", network.mac_address},
        {"network", network.network_name},
    };
}

auto from_json(const json_t& json, instance_t::network_t& network) //
    -> void
{
    json.at("ipAddress").get_to(network.ip_address);
    json.at("macAddress").get_to(network.mac_address);
    json.at("network").get_to(network.network_name);
}

auto to_json(json_t& json, const instance_t& instance) //
    -> void
{
    auto app_key = apps::key_t{instance.app_name().data(), instance.app_version().data()};
    json = json_t(
        {{"_schemaVersion", "2.0.0"},
         {"instanceId", instance.id().hex()},
         {"instanceName", instance.instance_name()},
         {"appKey", app_key},
         {"status", to_string(instance.status())},
         {"desired", to_string(instance.desired())},
         {"networks", instance.networks()},
         {"startupOptions", instance.startup_options()},
         {"usbDevices", instance.usb_devices()}});
}

auto from_json_v1(const json_t& j, instance_t& instance) //
    -> void
{
    instance._id = instance_id_t{j.at("id").get<std::string_view>()};
    instance._instance_name = j.at("instanceName").get<std::string>();
    instance._app_name = j.at("app").get<std::string>();
    instance._app_version = j.at("version").get<std::string>();
    instance._status = instance_status_from_string(j.at("status").get<std::string_view>());
    instance._desired = instance_status_from_string(j.at("desired").get<std::string_view>());
    instance._networks = j.at("networks").get<decltype(instance._networks)>();
    instance._startup_options = j.at("startupOptions").get<decltype(instance._startup_options)>();
    instance._usb_devices = j.at("usbDevices").get<decltype(instance._usb_devices)>();
}

auto from_json_v2(const json_t& j, instance_t& instance) //
    -> void
{
    instance._id = instance_id_t{j.at("instanceId").get<std::string_view>()};
    instance._instance_name = j.at("instanceName").get<std::string>();
    instance._app_name = j.at("appKey").at("name").get<std::string>();
    instance._app_version = j.at("appKey").at("version").get<std::string>();
    instance._status = instance_status_from_string(j.at("status").get<std::string_view>());
    instance._desired = instance_status_from_string(j.at("desired").get<std::string_view>());
    instance._networks = j.at("networks").get<decltype(instance._networks)>();
    instance._startup_options = j.at("startupOptions").get<decltype(instance._startup_options)>();
    instance._usb_devices = j.at("usbDevices").get<decltype(instance._usb_devices)>();
}

auto from_json(const json_t& j, instance_t& instance) //
    -> void
{
    auto schema_version = std::string_view{"1.0.0"};
    try {
        j.at("_schemaVersion").get_to(schema_version);
    } catch (...) {
    }

    try {
        schema_version[0] == '1' ? from_json_v1(j, instance) : from_json_v2(j, instance);
    } catch (...) {
        instance = instance_t{};
    }
}

auto operator==(const instance_t& lhs, const instance_t& rhs) //
    -> bool
{
    return (lhs.id() == rhs.id());
}

} // namespace flecs
