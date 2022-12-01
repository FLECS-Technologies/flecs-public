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

#include "deployment.h"

#include <cassert>
#include <fstream>
#include <map>
#include <regex>
#include <set>

#include "util/network/network.h"

namespace FLECS {

auto deployment_t::deployment_id() const noexcept //
    -> std::string_view
{
    return do_deployment_id();
}

#ifdef FLECS_UNIT_TEST
auto deployment_t::load(fs::path base_path) //
    -> result_t
{
    return do_load(base_path);
}

auto deployment_t::save(fs::path base_path) //
    -> result_t
{
    return do_save(base_path);
}
#endif // FLECS_UNIT_TEST

auto deployment_t::load() //
    -> result_t
{
    const auto base_path = "/var/lib/flecs/deployment/";
    return do_load(base_path);
}

auto deployment_t::save() //
    -> result_t
{
    const auto base_path = "/var/lib/flecs/deployment/";
    return do_save(base_path);
}

auto deployment_t::instances() const noexcept //
    -> const std::map<std::string, instance_t>&
{
    return _instances;
}

auto deployment_t::instances() noexcept //
    -> std::map<std::string, instance_t>&
{
    return _instances;
}

auto deployment_t::instance_ids(std::string_view app) const //
    -> std::vector<std::string>
{
    return instance_ids(app_key_t{app, ""}, AllVersions);
}

auto deployment_t::instance_ids(std::string_view app, std::string_view version) const //
    -> std::vector<std::string>
{
    return instance_ids(app_key_t{app, version}, MatchVersion);
}

auto deployment_t::instance_ids(const app_key_t& app_key, version_filter_e version_filter) const //
    -> std::vector<std::string>
{
    auto ids = std::vector<std::string>{};
    for (const auto& instance : instances())
    {
        if ((instance.second.app_name() == std::get<0>(app_key)) &&
            ((version_filter == AllVersions) || (instance.second.app_version() == std::get<1>(app_key))))
        {
            ids.emplace_back(instance.first);
        }
    }

    return ids;
}

auto deployment_t::instance_ids(const app_t& app, version_filter_e version_filter) const //
    -> std::vector<std::string>
{
    return instance_ids(app_key_t{app.app(), app.version()}, version_filter);
}

auto deployment_t::has_instance(std::string_view instance_id) const noexcept //
    -> bool
{
    return _instances.count(instance_id.data());
}

auto deployment_t::insert_instance(instance_t instance) //
    -> result_t
{
    _instances.emplace(instance.id(), instance);
    return do_insert_instance(std::move(instance));
}

auto deployment_t::create_instance(const app_t& app, std::string instance_name) //
    -> result_t
{
    // Step 1: Create unique id and insert instance
    auto tmp = instance_t{&app, instance_name, instance_status_e::REQUESTED, instance_status_e::CREATED};
    while (_instances.count(tmp.id()))
    {
        tmp.regenerate_id();
    }

    auto& instance = _instances.emplace(tmp.id(), tmp).first->second;
    for (const auto& startup_option : app.startup_options())
    {
        instance.startup_options().emplace_back(static_cast<std::underlying_type_t<startup_option_t>>(startup_option));
    }

    // Step 2: Create volumes
    const auto [res, additional_info] = create_volumes(instance);
    if (res != 0)
    {
        return {res, additional_info};
    }

    // Step 3: Create networks
    // query and create default network, if required
    const auto network_name = default_network_name();
    if (!network_name.empty())
    {
        const auto network = query_network(network_name);
        if (!network.has_value())
        {
            const auto [res, additional_info] = create_network(
                default_network_type(),
                default_network_name(),
                default_network_cidr_subnet(),
                default_network_gateway(),
                {});
            if (res != 0)
            {
                return {-1, instance.id()};
            }
        }
        instance.networks().emplace_back(instance_t::network_t{
            .network_name = default_network_name().data(),
            .mac_address = app.networks()[0].mac_address(),
            .ip_address = {},
        });
    }

#if 0  // Additional networks are experimental and untested - disable for now
    // query and create remaining networks
    for (const auto& network : app.networks())
    {
        const auto network_exists = query_network(network.name()).has_value();
        if (!network_exists)
        {
            const auto [res, err] = create_network(network.type(), network.name(), "", "", network.parent());
            if (res != 0)
            {
                return {-1, instance.id()};
            }
        }
    }
#endif // 0

    // Step 4: Create conffiles
    const auto container_name = std::string{"flecs-"} + instance.id();
    const auto conf_path = std::string{"/var/lib/flecs/instances/"} + instance.id() + std::string{"/conf/"};
    if (!app.conffiles().empty())
    {
        auto ec = std::error_code{};
        if (!std::filesystem::create_directories(conf_path, ec))
        {
            return {-1, instance.id()};
        }
    }

    for (const auto& conffile : app.conffiles())
    {
        const auto local_path = conf_path + conffile.local();
        if (conffile.init())
        {
            const auto [res, additional_info] =
                copy_file_from_image(app.image_with_tag(), conffile.container(), local_path);
            if (res != 0)
            {
                return {-1, instance.id()};
            }
        }
        else
        {
            auto f = std::ofstream{local_path};
            if (!f.good())
            {
                return {-1, instance.id()};
            }
        }
    }

    instance.status(instance_status_e::RESOURCES_READY);

    return do_create_instance(app, instance);
}

auto deployment_t::delete_instance(std::string_view instance_id) //
    -> result_t
{
    const auto [res, additional_info] = do_delete_instance(std::move(instance_id));
    _instances.erase(instance_id.data());
    return {res, additional_info};
}

auto deployment_t::start_instance(std::string_view instance_id) //
    -> result_t
{
    auto& instance = _instances.at(instance_id.data());
    const auto& startup_options = instance.startup_options();

    if (std::count(
            startup_options.cbegin(),
            startup_options.cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option_t::INIT_NETWORK_AFTER_START)))
    {
        for (const auto& network : instance.networks())
        {
            disconnect_network(instance_id, network.network_name);
        }
    }

    const auto [res, additional_info] = do_start_instance(instance);

    if (res != 0)
    {
        return {res, additional_info};
    }

    if (std::count(
            startup_options.cbegin(),
            startup_options.cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option_t::INIT_NETWORK_AFTER_START)))
    {
        for (const auto& network : _instances.at(instance_id.data()).networks())
        {
            connect_network(instance_id, network.network_name, network.ip_address);
        }
    }

    return ready_instance(instance_id);
}

auto deployment_t::ready_instance(std::string_view instance_id) //
    -> result_t
{
    const auto& instance = _instances.at(instance_id.data());
    return do_ready_instance(instance);
}

auto deployment_t::stop_instance(std::string_view instance_id) //
    -> result_t
{
    const auto& instance = _instances.at(instance_id.data());
    const auto& startup_options = instance.startup_options();

    auto [res, additional_info] = do_stop_instance(instance);

    if (std::count(
            startup_options.cbegin(),
            startup_options.cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option_t::INIT_NETWORK_AFTER_START)))
    {
        for (const auto& network : _instances.at(instance_id.data()).networks())
        {
            const auto [net_res, net_err] = disconnect_network(instance_id, network.network_name);
            if (net_res != 0)
            {
                res = -1;
                additional_info += '\n' + net_err;
            }
        }
    }

    return {res, additional_info};
}

auto deployment_t::is_instance_runnable(std::string_view instance_id) const //
    -> bool
{
    return has_instance(instance_id) && (instances().at(instance_id.data()).status() == instance_status_e::CREATED);
}

auto deployment_t::is_instance_running(std::string_view instance_id) const //
    -> bool
{
    if (!has_instance(instance_id))
    {
        return false;
    }
    return do_is_instance_running(instances().at(instance_id.data()));
}

auto deployment_t::create_network(
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

auto deployment_t::query_network(std::string_view network) //
    -> std::optional<network_t>
{
    return do_query_network(std::move(network));
}

auto deployment_t::delete_network(std::string_view network) //
    -> result_t
{
    return do_delete_network(std::move(network));
}

auto deployment_t::connect_network(
    std::string_view instance_id,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    return do_connect_network(std::move(instance_id), std::move(network), std::move(ip));
}

auto deployment_t::disconnect_network(std::string_view instance_id, std::string_view network) //
    -> result_t
{
    return do_disconnect_network(std::move(instance_id), std::move(network));
}

auto deployment_t::create_volumes(const instance_t& instance) //
    -> result_t
{
    for (auto& volume : instance.app().volumes())
    {
        if (volume.type() == volume_t::VOLUME)
        {
            const auto [res, additional_info] = create_volume(instance.id(), volume.host());
            if (res != 0)
            {
                return {res, additional_info};
            }
        }
    }
    return {0, {}};
}

auto deployment_t::create_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_create_volume(std::move(instance_id), std::move(volume_name));
}

auto deployment_t::export_volumes(const instance_t& instance, fs::path dest_dir) //
    -> result_t
{
    for (auto& volume : instance.app().volumes())
    {
        const auto [res, additional_info] = export_volume(instance, volume.host(), dest_dir);
        if (res != 0)
        {
            return {res, additional_info};
        }
    }

    return {0, {}};
}

auto deployment_t::export_volume(const instance_t& instance, std::string_view volume_name, fs::path dest_dir) //
    -> result_t
{
    return do_export_volume(instance, std::move(volume_name), std::move(dest_dir));
}

auto deployment_t::delete_volume(std::string_view instance_id, std::string_view volume_name) //
    -> result_t
{
    return do_delete_volume(std::move(instance_id), std::move(volume_name));
}

auto deployment_t::copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_from_image(image, file, dest);
}

auto deployment_t::copy_file_to_instance(std::string_view instance_id, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_to_instance(instance_id, file, dest);
}

auto deployment_t::copy_file_from_instance(std::string_view instance_id, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_from_instance(instance_id, file, dest);
}

auto deployment_t::default_network_name() const //
    -> std::string_view
{
    return do_default_network_name();
}

auto deployment_t::default_network_type() const //
    -> network_type_t
{
    return do_default_network_type();
}

auto deployment_t::default_network_cidr_subnet() const //
    -> std::string_view
{
    return do_default_network_cidr_subnet();
}

auto deployment_t::default_network_gateway() const //
    -> std::string_view
{
    return do_default_network_gateway();
}

auto deployment_t::generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
    -> std::string
{
    // parse a.b.c.d
    auto base_ip = in_addr_t{};
    {
        // from beginning of line: (a.b.c.)(d/)
        // e.g. 127.0.0.1/24 -> (127.0.0.)(1)/
        const auto ip_regex = std::regex{R"((^(?:\d{1,3}\.){3}\d{1,3})\/)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, ip_regex))
        {
            return std::string{};
        }
        base_ip = ntohl(ipv4_to_bits(m[1].str()).s_addr);
    }
    // parse /x
    auto subnet_size = int{};
    {
        // until end of line: d/[0-32]
        // e.g. 127.0.0.1/24 -> 1/24
        const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2)
        {
            return std::string{};
        }
        subnet_size = std::stoi(m[1].str());
    }

    // determine the last ip address that belongs to the subnet:
    // (~0u << subnet_size) creates an inverted bitmask (such as 0xff00),
    // which is again inverted (yielding e.g. 0x00ff) and or'ed with the base ip.
    // finally subtract 1 to exclude the network's broadcast address
    const auto max_ip = (base_ip | ~(~0u << subnet_size)) - 1;

    auto used_ips = std::set<in_addr_t>{};
    if (!gateway.empty())
    {
        used_ips.emplace(ntohl(ipv4_to_bits(gateway).s_addr));
    }
    for (const auto& instance : _instances)
    {
        for (const auto& network : instance.second.networks())
        {
            used_ips.emplace(ntohl(ipv4_to_bits(network.ip_address).s_addr));
        }
    }

    // skip network address and host address
    auto instance_ip = base_ip + 2;

    // search first unused address
    while (used_ips.find(instance_ip) != used_ips.end())
    {
        ++instance_ip;
    }

    if (instance_ip > max_ip)
    {
        return std::string{};
    }

    return ipv4_to_string(in_addr{.s_addr = htonl(instance_ip)});
}

auto deployment_t::do_load(fs::path base_path) //
    -> result_t
{
    using std::operator""sv;

    auto json_file = std::ifstream{base_path / deployment_id().data() += ".json"sv};
    if (!json_file.good())
    {
        return {-1, "Could not open json"};
    }

    auto instances_json = parse_json(json_file);
    instances_json.get_to(_instances);

    return {0, {}};
}

auto deployment_t::do_save(fs::path base_path) //
    -> result_t
{
    auto ec = std::error_code{};
    fs::create_directories(base_path, ec);
    if (ec)
    {
        return {-1, "Could not create directory"};
    }

    base_path /= deployment_id().data();
    base_path += ".json.new";

    auto instances_json = std::ofstream{base_path, std::ios_base::out | std::ios_base::trunc};
    try
    {
        instances_json << json_t(instances());
        instances_json.flush();

        const auto from = base_path;
        const auto to = base_path.replace_extension();
        fs::rename(from, to);
    }
    catch (const std::exception& ex)
    {
        return {-1, ex.what()};
    }

    return {0, {}};
}

} // namespace FLECS
