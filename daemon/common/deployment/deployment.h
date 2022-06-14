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

#ifndef BD6EE81F_EC42_4122_8EE7_5036BA499377
#define BD6EE81F_EC42_4122_8EE7_5036BA499377

#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <tuple>

#include "core/flecs.h"
#include "instance/instance.h"

namespace FLECS {

class app_t;

enum class network_type_t
{
    NONE,
    INTERNAL,
    BRIDGE,
    MACVLAN,
    IPVLAN,
};

auto to_string(const network_type_t& network_type) //
    -> std::string;
auto network_type_from_string(std::string_view str) //
    -> network_type_t;

class deployment_t
{
public:
    struct network_t
    {
        std::string name;
        std::string cidr_subnet;
        std::string gateway;
        network_type_t type;
    };

    deployment_t() = default;

    virtual ~deployment_t() = default;

    auto instances() const noexcept //
        -> const std::map<std::string, instance_t>&;
    auto insert_instance(instance_t instance) //
        -> result_t;
    auto create_instance(const app_t& app) //
        -> result_t;
    auto delete_instance(std::string_view instance_id) //
        -> result_t;
    auto start_instance(std::string_view instance_id) //
        -> result_t;
    auto ready_instance(std::string_view instance_id) //
        -> result_t;
    auto stop_instance(std::string_view instance_id) //
        -> result_t;
    auto create_network(
        network_type_t network_type,
        std::string_view network,
        std::string_view cidr_subnet,
        std::string_view gateway,
        std::string_view parent_adapter) //
        -> result_t;
    auto query_network(std::string_view network) //
        -> std::optional<network_t>;
    auto delete_network(std::string_view network) //
        -> result_t;
    auto connect_network(
        std::string_view instance_id,
        std::string_view network,
        std::string_view ip) //
        -> result_t;
    auto disconnect_network(std::string_view instance_id, std::string_view network) //
        -> result_t;
    auto create_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t;
    auto delete_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t;

    auto generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
        -> std::string;

protected:
    std::map<std::string, instance_t> _instances;

private:
    virtual auto do_insert_instance(instance_t instance) //
        -> result_t = 0;
    virtual auto do_create_instance(const app_t& app) //
        -> result_t = 0;
    virtual auto do_delete_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_start_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_ready_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_stop_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_create_network(
        network_type_t network_type,
        std::string_view network,
        std::string_view cidr_subnet,
        std::string_view gateway,
        std::string_view parent_adapter) //
        -> result_t = 0;
    virtual auto do_query_network(std::string_view network) //
        -> std::optional<network_t> = 0;
    virtual auto do_delete_network(std::string_view network) //
        -> result_t = 0;
    virtual auto do_connect_network(
        std::string_view instance_id,
        std::string_view network,
        std::string_view ip) //
        -> result_t = 0;
    virtual auto do_disconnect_network(std::string_view instance_id, std::string_view network) //
        -> result_t = 0;
    virtual auto do_create_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t = 0;
    virtual auto do_delete_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t = 0;
};

inline auto deployment_t::instances() const noexcept //
    -> const std::map<std::string, instance_t>&
{
    return _instances;
}

inline auto deployment_t::insert_instance(instance_t instance) //
    -> result_t
{
    return do_insert_instance(instance);
}

inline auto deployment_t::create_instance(const app_t& app) //
    -> result_t
{
    return do_create_instance(app);
}

inline auto deployment_t::ready_instance(std::string_view instance_id) //
    -> result_t
{
    return do_ready_instance(instance_id);
}

inline auto deployment_t::delete_instance(std::string_view instance_id) //
    -> result_t
{
    return do_delete_instance(instance_id);
}

inline auto deployment_t::create_network(
    network_type_t network_type,
    std::string_view network,
    std::string_view cidr_subnet,
    std::string_view gateway,
    std::string_view parent_adapter) //
    -> result_t
{
    return do_create_network(network_type, network, cidr_subnet, gateway, parent_adapter);
}

inline auto deployment_t::delete_network(std::string_view network) //
    -> result_t
{
    return do_delete_network(network);
}

inline auto deployment_t::query_network(std::string_view network) //
    -> std::optional<network_t>
{
    return do_query_network(network);
}

inline auto deployment_t::connect_network(
    std::string_view instance_id,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    return do_connect_network(instance_id, network, ip);
}

inline auto deployment_t::disconnect_network(std::string_view instance_id, std::string_view network) //
    -> result_t
{
    return do_disconnect_network(instance_id, network);
}

inline auto deployment_t::create_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_create_volume(instance_id, volume_name);
}

inline auto deployment_t::delete_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_delete_volume(instance_id, volume_name);
}

} // namespace FLECS

#endif // BD6EE81F_EC42_4122_8EE7_5036BA499377
