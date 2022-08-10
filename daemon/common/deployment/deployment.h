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
#include "util/fs/fs.h"

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
        std::string parent;
        network_type_t type;
    };

    deployment_t() = default;

    virtual ~deployment_t() = default;

    auto deployment_id() const noexcept //
        -> std::string_view;

    auto load() //
        -> result_t;
    auto save() //
        -> result_t;

    auto instances() noexcept //
        -> std::map<std::string, instance_t>&;
    auto instances() const noexcept //
        -> const std::map<std::string, instance_t>&;
    auto instance_ids(std::string_view app) const //
        -> std::vector<std::string>;
    auto instance_ids(std::string_view app, std::string_view version) const //
        -> std::vector<std::string>;
    auto has_instance(std::string_view instance_id) const noexcept //
        -> bool;
    auto insert_instance(instance_t instance) //
        -> result_t;
    auto create_instance(const app_t& app, std::string instance_name) //
        -> result_t;
    auto delete_instance(std::string_view instance_id) //
        -> result_t;
    auto start_instance(const app_t& app, std::string_view instance_id) //
        -> result_t;
    auto ready_instance(std::string_view instance_id) //
        -> result_t;
    auto stop_instance(std::string_view instance_id) //
        -> result_t;
    auto is_instance_runnable(std::string_view instance_id) const //
        -> bool;
    auto is_instance_running(std::string_view instance_id) const //
        -> bool;
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
    auto copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t;
    auto default_network_name() const //
        -> std::string_view;
    auto default_network_type() const //
        -> network_type_t;
    auto default_network_cidr_subnet() const //
        -> std::string_view;
    auto default_network_gateway() const //
        -> std::string_view;

    auto generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
        -> std::string;

protected:
    std::map<std::string, instance_t> _instances;
    std::map<std::string, network_t> _networks;

private:
    virtual auto do_deployment_id() const noexcept //
        -> std::string_view = 0;

    virtual auto do_insert_instance(instance_t instance) //
        -> result_t = 0;
    virtual auto do_create_instance(const app_t& app, instance_t& instance) //
        -> result_t = 0;
    virtual auto do_delete_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_start_instance(const app_t& app, instance_t& instance) //
        -> result_t = 0;
    virtual auto do_ready_instance(const instance_t& instance) //
        -> result_t = 0;
    virtual auto do_stop_instance(const instance_t& instance) //
        -> result_t = 0;
    virtual auto do_is_instance_running(const instance_t& instance) const //
        -> bool = 0;
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
    virtual auto do_copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t = 0;
    virtual auto do_default_network_name() const //
        -> std::string_view = 0;
    virtual auto do_default_network_type() const //
        -> network_type_t = 0;
    virtual auto do_default_network_cidr_subnet() const //
        -> std::string_view = 0;
    virtual auto do_default_network_gateway() const //
        -> std::string_view = 0;
};

inline auto deployment_t::instances() const noexcept //
    -> const std::map<std::string, instance_t>&
{
    return _instances;
}

inline auto deployment_t::instances() noexcept //
    -> std::map<std::string, instance_t>&
{
    return _instances;
}

inline auto deployment_t::instance_ids(std::string_view app) const //
    -> std::vector<std::string>
{
    auto ids = std::vector<std::string>{};
    for (const auto& instance : instances())
    {
        if (app == instance.second.app())
        {
            ids.emplace_back(instance.first);
        }
    }
    return ids;
}

inline auto deployment_t::instance_ids(std::string_view app, std::string_view version) const //
    -> std::vector<std::string>
{
    auto ids = std::vector<std::string>{};
    for (const auto& instance : instances())
    {
        if ((app == instance.second.app()) && (version == instance.second.version()))
        {
            ids.emplace_back(instance.first);
        }
    }
    return ids;
}

inline auto deployment_t::has_instance(std::string_view instance_id) const noexcept //
    -> bool
{
    return _instances.count(instance_id.data());
}

inline auto deployment_t::insert_instance(instance_t instance) //
    -> result_t
{
    _instances.emplace(instance.id(), instance);
    return do_insert_instance(std::move(instance));
}

inline auto deployment_t::delete_instance(std::string_view instance_id) //
    -> result_t
{
    const auto [res, additional_info] = do_delete_instance(std::move(instance_id));
    _instances.erase(instance_id.data());
    return {res, additional_info};
}

inline auto deployment_t::is_instance_running(std::string_view instance_id) const //
    -> bool
{
    if (!has_instance(instance_id))
    {
        return false;
    }
    return do_is_instance_running(instances().at(instance_id.data()));
}

inline auto deployment_t::create_network(
    network_type_t network_type,
    std::string_view network,
    std::string_view cidr_subnet,
    std::string_view gateway,
    std::string_view parent_adapter) //
    -> result_t
{
    return do_create_network(
        std::move(network_type),
        std::move(network),
        std::move(cidr_subnet),
        std::move(gateway),
        std::move(parent_adapter));
}

inline auto deployment_t::delete_network(std::string_view network) //
    -> result_t
{
    return do_delete_network(std::move(network));
}

inline auto deployment_t::query_network(std::string_view network) //
    -> std::optional<network_t>
{
    return do_query_network(std::move(network));
}

inline auto deployment_t::connect_network(
    std::string_view instance_id,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    return do_connect_network(std::move(instance_id), std::move(network), std::move(ip));
}

inline auto deployment_t::disconnect_network(std::string_view instance_id, std::string_view network) //
    -> result_t
{
    return do_disconnect_network(std::move(instance_id), std::move(network));
}

inline auto deployment_t::create_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_create_volume(std::move(instance_id), std::move(volume_name));
}

inline auto deployment_t::delete_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_delete_volume(std::move(instance_id), std::move(volume_name));
}

inline auto deployment_t::copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_from_image(image, file, dest);
}

inline auto deployment_t::default_network_name() const //
    -> std::string_view
{
    return do_default_network_name();
}

inline auto deployment_t::default_network_type() const //
    -> network_type_t
{
    return do_default_network_type();
}

inline auto deployment_t::default_network_cidr_subnet() const //
    -> std::string_view
{
    return do_default_network_cidr_subnet();
}

inline auto deployment_t::default_network_gateway() const //
    -> std::string_view
{
    return do_default_network_gateway();
}

} // namespace FLECS

#endif // BD6EE81F_EC42_4122_8EE7_5036BA499377
