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

#include "instance.h"

#include <cstdio>
#include <random>

namespace FLECS {

std::string generate_instance_id()
{
    auto res = std::string(8, '\0');

    auto seed = std::random_device{};
    auto generator = std::mt19937{seed()};
    auto distribution = std::uniform_int_distribution{
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()};

    auto id = distribution(generator);
    std::snprintf(res.data(), res.length() + 1, "%.8x", id);

    return res;
}

instance_t::instance_t()
    : instance_t{generate_instance_id(), {}, {}, {}, {}, {}}
{}

instance_t::instance_t(
    std::string app,
    std::string version,
    std::string instance_name,
    instance_status_e status,
    instance_status_e desired)
    : instance_t{generate_instance_id(), app, version, instance_name, status, desired}
{}

instance_t::instance_t(
    std::string id,
    std::string app,
    std::string version,
    std::string instance_name,
    instance_status_e status,
    instance_status_e desired)
    : _id{id}
    , _app{app}
    , _version{version}
    , _instance_name{instance_name}
    , _status{status}
    , _desired{desired}
    , _networks{}
    , _startup_options{}
{}

auto instance_t::regenerate_id() //
    -> void
{
    _id = generate_instance_id();
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
    json = json_t(
        {{"app", instance._app},
         {"desired", to_string(instance._desired)},
         {"id", instance._id},
         {"instanceName", instance._instance_name},
         {"networks", instance._networks},
         {"startupOptions", instance._startup_options},
         {"status", to_string(instance._status)},
         {"version", instance._version}});
}

auto from_json(const json_t& json, instance_t& instance) //
    -> void
{
    json.at("app").get_to(instance._app);
    auto desired = std::string{};
    json.at("desired").get_to(desired);
    instance._desired = instance_status_from_string(desired);
    json.at("id").get_to(instance._id);
    json.at("instanceName").get_to(instance._instance_name);
    json.at("networks").get_to(instance._networks);
    json.at("startupOptions").get_to(instance._startup_options);
    auto status = std::string{};
    json.at("status").get_to(status);
    instance._status = instance_status_from_string(status);
    json.at("version").get_to(instance._version);
}

} // namespace FLECS
