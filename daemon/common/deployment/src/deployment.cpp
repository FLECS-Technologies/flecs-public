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

#include "deployment.h"

#include <algorithm>
#include <fstream>
#include <map>
#include <regex>
#include <set>

#include "common/app/app.h"
#include "common/app/manifest/manifest.h"
#include "util/network/ip_addr.h"
#include "util/network/network.h"

namespace FLECS {

auto deployment_t::deployment_id() const noexcept //
    -> std::string_view
{
    return do_deployment_id();
}

auto deployment_t::load(const fs::path& base_path) //
    -> result_t
{
    return do_load(base_path);
}

auto deployment_t::save(const fs::path& base_path) //
    -> result_t
{
    return do_save(base_path);
}

auto deployment_t::instance_ids() const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{});
}

auto deployment_t::instance_ids(std::string_view app) const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{app.data(), ""});
}

auto deployment_t::instance_ids(std::string_view app, std::string_view version) const //
    -> std::vector<instance_id_t>
{
    return instance_ids(app_key_t{app.data(), version.data()});
}

auto deployment_t::instance_ids(const app_key_t& app_key) const //
    -> std::vector<instance_id_t>
{
    auto ids = std::vector<instance_id_t>{};
    for (const auto& instance : _instances) {
        const auto apps_match = app_key.name().empty() || (app_key.name() == instance->app_name());
        const auto versions_match = app_key.name().empty() || app_key.version().empty() ||
                                    (app_key.version() == instance->app_version());
        if (apps_match && versions_match) {
            ids.emplace_back(instance->id());
        }
    }

    return ids;
}

auto deployment_t::query_instance(instance_id_t instance_id) const //
    -> std::shared_ptr<instance_t>
{
    const auto it = std::find_if(
        _instances.cbegin(),
        _instances.cend(),
        [&instance_id](const std::shared_ptr<instance_t>& elem) {
            return elem->id() == instance_id;
        });
    return it != _instances.cend() ? *it : nullptr;
}

auto deployment_t::has_instance(instance_id_t instance_id) const noexcept //
    -> bool
{
    const auto it = std::find_if(
        _instances.cbegin(),
        _instances.cend(),
        [&instance_id](const std::shared_ptr<instance_t>& elem) {
            return elem->id() == instance_id;
        });
    return it != _instances.cend();
}

auto deployment_t::insert_instance(instance_t instance) //
    -> std::shared_ptr<instance_t>
{
    return _instances.emplace_back(std::make_shared<instance_t>(std::move(instance)));
}

auto deployment_t::create_instance(std::shared_ptr<const app_t> app, std::string instance_name) //
    -> result_t
{
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    // Step 1: Create unique id and insert instance
    auto tmp = instance_t{app, instance_name};
    while (has_instance(tmp.id())) {
        tmp.regenerate_id();
    }

    tmp.status(instance_status_e::Requested);
    tmp.desired(instance_status_e::Created);

    auto instance = insert_instance(std::move(tmp));
    for (const auto& startup_option : manifest->startup_options()) {
        instance->startup_options().emplace_back(
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option));
    }

    // Step 2: Create volumes
    {
        auto [res, additional_info] = create_volumes(instance);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    // Step 3: Create networks
    // query and create default network, if required
    const auto network_name = default_network_name();
    if (!network_name.empty()) {
        const auto network = query_network(network_name);
        if (!network.has_value()) {
            const auto [res, additional_info] = create_network(
                default_network_type(),
                default_network_name(),
                default_network_cidr_subnet(),
                default_network_gateway(),
                {});
            if (res != 0) {
                return {-1, instance->id().hex()};
            }
        }
        instance->networks().emplace_back(instance_t::network_t{
            .network_name = default_network_name().data(),
            .mac_address = manifest->networks()[0].mac_address(),
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
    {
        auto [res, additional_info] = create_config_files(instance);
        if (res != 0) {
            return {res, instance->id().hex()};
        }
        instance->status(instance_status_e::ResourcesReady);
    }

    return do_create_instance(instance);
}

auto deployment_t::delete_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    const auto [res, additional_info] = do_delete_instance(instance);
    _instances.erase(
        std::remove_if(
            _instances.begin(),
            _instances.end(),
            [&instance](const std::shared_ptr<instance_t>& elem) {
                return elem->id() == instance->id();
            }),
        _instances.end());
    return {res, additional_info};
}

auto deployment_t::start_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    if (std::count(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(
                startup_option_t::INIT_NETWORK_AFTER_START))) {
        for (const auto& network : instance->networks()) {
            disconnect_network(instance, network.network_name);
        }
    }

    const auto [res, additional_info] = do_start_instance(instance);

    if (res != 0) {
        return {res, additional_info};
    }

    if (std::count(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(
                startup_option_t::INIT_NETWORK_AFTER_START))) {
        for (const auto& network : instance->networks()) {
            connect_network(instance, network.network_name, network.ip_address);
        }
    }

    return ready_instance(std::move(instance));
}

auto deployment_t::ready_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    return do_ready_instance(std::move(instance));
}

auto deployment_t::stop_instance(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    auto [res, additional_info] = do_stop_instance(instance);

    if (std::count(
            instance->startup_options().cbegin(),
            instance->startup_options().cend(),
            static_cast<std::underlying_type_t<startup_option_t>>(
                startup_option_t::INIT_NETWORK_AFTER_START))) {
        for (const auto& network : instance->networks()) {
            const auto [net_res, net_err] = disconnect_network(instance, network.network_name);
            if (net_res != 0) {
                res = -1;
                additional_info += '\n' + net_err;
            }
        }
    }

    return {res, additional_info};
}

auto deployment_t::export_instance(std::shared_ptr<instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    dest_dir /= instance->id().hex();
    auto ec = std::error_code{};
    fs::create_directories(dest_dir, ec);
    if (ec) {
        return {-1, "Could not create export directory"};
    }

    auto [res, additional_info] = export_volumes(instance, dest_dir / "volumes");
    if (res != 0) {
        return {res, additional_info};
    }

    std::tie(res, additional_info) = export_config_files(instance, dest_dir / "conf");
    if (res != 0) {
        return {res, additional_info};
    }

    return do_export_instance(instance, dest_dir);
}

auto deployment_t::import_instance(std::shared_ptr<instance_t> instance, fs::path base_dir) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    base_dir /= instance->id().hex();

    auto [res, additional_info] = import_volumes(instance, base_dir / "volumes");
    if (res != 0) {
        return {res, additional_info};
    }

    std::tie(res, additional_info) = import_config_files(instance, base_dir / "conf");
    if (res != 0) {
        return {res, additional_info};
    }

    return do_import_instance(instance, base_dir);
}

auto deployment_t::is_instance_runnable(std::shared_ptr<instance_t> instance) const //
    -> bool
{
    return instance && instance->status() == instance_status_e::Created;
}

auto deployment_t::is_instance_running(std::shared_ptr<instance_t> instance) const //
    -> bool
{
    return instance && do_is_instance_running(std::move(instance));
}

auto deployment_t::create_config_files(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
    if (!manifest->conffiles().empty()) {
        auto ec = std::error_code{};
        std::filesystem::create_directories(conf_path, ec);
        if (ec) {
            return {-1, instance->id().hex()};
        }
    }

    for (const auto& conffile : manifest->conffiles()) {
        const auto local_path = conf_path + conffile.local();
        if (conffile.init()) {
            const auto [res, additional_info] =
                copy_file_from_image(manifest->image_with_tag(), conffile.container(), local_path);
            if (res != 0) {
                return {-1, instance->id().hex()};
            }
        } else {
            auto f = std::ofstream{local_path};
            if (!f.good()) {
                return {-1, instance->id().hex()};
            }
        }
    }

    return {0, {}};
}

auto deployment_t::create_network(
    network_type_e network_type,
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
    std::shared_ptr<instance_t> instance,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    return do_connect_network(std::move(instance), std::move(network), std::move(ip));
}

auto deployment_t::disconnect_network(
    std::shared_ptr<instance_t> instance, std::string_view network) //
    -> result_t
{
    return do_disconnect_network(std::move(instance), std::move(network));
}

auto deployment_t::create_volumes(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& volume : manifest->volumes()) {
        if (volume.type() == volume_t::VOLUME) {
            const auto [res, additional_info] = create_volume(instance, volume.host());
            if (res != 0) {
                return {res, additional_info};
            }
        }
    }
    return {0, {}};
}

auto deployment_t::create_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    return do_create_volume(std::move(instance), std::move(volume_name));
}

auto deployment_t::import_volumes(std::shared_ptr<instance_t> instance, fs::path src_dir) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& volume : manifest->volumes()) {
        if (volume.type() == volume_t::VOLUME) {
            const auto [res, additional_info] = import_volume(instance, volume.host(), src_dir);
            if (res != 0) {
                return {res, additional_info};
            }
        }
    }

    return {0, {}};
}

auto deployment_t::import_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name, fs::path src_dir) //
    -> result_t
{
    auto ec = std::error_code{};
    if (!fs::exists(src_dir, ec) || !fs::is_directory(src_dir, ec)) {
        return {-1, "Source directory does not exist"};
    }

    return do_import_volume(std::move(instance), volume_name, src_dir);
}

auto deployment_t::export_volumes(std::shared_ptr<instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& volume : manifest->volumes()) {
        const auto [res, additional_info] = export_volume(instance, volume.host(), dest_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    return {0, {}};
}

auto deployment_t::export_volume(
    std::shared_ptr<instance_t> instance,
    std::string_view volume_name,
    fs::path dest_dir) const //
    -> result_t
{
    auto ec = std::error_code{};
    fs::create_directories(dest_dir, ec);
    if (ec) {
        return {-1, "Could not create export directory"};
    }

    return do_export_volume(std::move(instance), std::move(volume_name), std::move(dest_dir));
}

auto deployment_t::export_config_files(
    std::shared_ptr<instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& config_file : manifest->conffiles()) {
        const auto [res, additional_info] = export_config_file(instance, config_file, dest_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    return {0, {}};
}
auto deployment_t::export_config_file(
    std::shared_ptr<instance_t> instance,
    const conffile_t& config_file,
    fs::path dest_dir) const //
    -> result_t
{
    if (is_instance_running(instance)) {
        const auto [res, additional_info] = copy_file_from_instance(
            instance,
            config_file.container(),
            dest_dir / "conf" / config_file.local());
        if (res != 0) {
            return {res, additional_info};
        }
    } else {
        const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
        /* copy config files from local dir for stopped instances */
        auto ec = std::error_code{};
        fs::copy(conf_path + config_file.local(), dest_dir / "conf" / config_file.local(), ec);
        if (ec) {
            return {-1, "Could not export conffile from local directory"};
        }
    }

    return {0, {}};
}

auto deployment_t::import_config_files(std::shared_ptr<instance_t> instance, fs::path base_dir) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& config_file : manifest->conffiles()) {
        const auto [res, additional_info] = import_config_file(instance, config_file, base_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    return {0, {}};
}

auto deployment_t::import_config_file(
    std::shared_ptr<instance_t> instance, const conffile_t& config_file, fs::path base_dir) //
    -> result_t
{
    const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
    /* copy config files from local dir for stopped instances */
    auto ec = std::error_code{};
    fs::copy(
        conf_path + config_file.local(),
        base_dir / "conf" / config_file.local(),
        fs::copy_options::overwrite_existing,
        ec);
    if (ec) {
        return {-1, "Could not import conffile"};
    }

    return {0, {}};
}

auto deployment_t::delete_volumes(std::shared_ptr<instance_t> instance) //
    -> result_t
{
    auto app = instance->app();
    if (!app) {
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    for (auto& volume : manifest->volumes()) {
        if (volume.type() == volume_t::VOLUME) {
            const auto [res, additional_info] = delete_volume(instance, volume.host());
            if (res != 0) {
                return {res, additional_info};
            }
        }
    }
    return {0, {}};
}

auto deployment_t::delete_volume(
    std::shared_ptr<instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    return do_delete_volume(std::move(instance), std::move(volume_name));
}

auto deployment_t::copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_from_image(image, file, dest);
}

auto deployment_t::copy_file_to_instance(
    std::shared_ptr<instance_t> instance, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_to_instance(std::move(instance), file, dest);
}

auto deployment_t::copy_file_from_instance(
    std::shared_ptr<instance_t> instance, fs::path file, fs::path dest) const //
    -> result_t
{
    return do_copy_file_from_instance(std::move(instance), file, dest);
}

auto deployment_t::default_network_name() const //
    -> std::string_view
{
    return do_default_network_name();
}

auto deployment_t::default_network_type() const //
    -> network_type_e
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

auto deployment_t::generate_instance_ip(
    std::string_view cidr_subnet, std::string_view gateway) const //
    -> std::string
{
    // parse a.b.c.d
    auto base_ip = ip_addr_t{};
    {
        // from beginning of line: (a.b.c.)(d/)
        // e.g. 127.0.0.1/24 -> (127.0.0.)(1)/
        const auto ip_regex = std::regex{R"((^(?:\d{1,3}\.){3}\d{1,3})\/)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, ip_regex)) {
            return {};
        }
        base_ip = ip_addr_t{m[1].str()};
    }
    // parse /x
    auto subnet_size = int{};
    {
        // until end of line: d/[0-32]
        // e.g. 127.0.0.1/24 -> 1/24
        const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2) {
            return {};
        }
        subnet_size = std::stoi(m[1].str());
    }

    // determine the last ip address that belongs to the subnet:
    // (~0u << subnet_size) creates an inverted bitmask (such as 0xff00),
    // which is again inverted (yielding e.g. 0x00ff) and or'ed with the base ip.
    // finally subtract 1 to exclude the network's broadcast address
    const auto max_ip = ip_addr_t{(base_ip.addr_v4().s_addr | ~(~0u << subnet_size)) - 1};

    auto used_ips = std::set<ip_addr_t>{};
    if (!gateway.empty()) {
        used_ips.emplace(gateway);
    }
    for (const auto& instance : _instances) {
        for (const auto& network : instance->networks()) {
            if (!network.ip_address.empty()) {
                used_ips.emplace(network.ip_address);
            }
        }
    }

    // skip network address and host address
    auto instance_ip = base_ip + 2;

    // search first unused address
    while (used_ips.count(instance_ip)) {
        ++instance_ip;
    }

    if (instance_ip > max_ip) {
        return {};
    }

    return to_string(instance_ip);
}

auto deployment_t::do_load(const fs::path& base_path) //
    -> result_t
{
    using std::operator""sv;

    auto json_file = std::ifstream{base_path / "deployment" / deployment_id().data() += ".json"sv};
    if (!json_file.good()) {
        return {-1, "Could not open json"};
    }

    auto instances_json = parse_json(json_file);
    try {
        _instances.reserve(instances_json.size());
        for (const auto& instance : instances_json) {
            _instances.push_back(std::make_shared<instance_t>(instance.get<instance_t>()));
        }
    } catch (const std::exception& ex) {
        _instances.clear();
        return {-1, ex.what()};
    }

    return {0, {}};
}

auto deployment_t::do_save(const fs::path& base_path) //
    -> result_t
{
    auto path = base_path / "deployment";
    auto ec = std::error_code{};
    fs::create_directories(path, ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }

    using std::operator""s;
    auto json_new = path / (deployment_id().data() + ".json.new"s);
    try {
        auto json_file = std::ofstream{json_new, std::ios_base::out | std::ios_base::trunc};
        auto instances_json = json_t::array();
        for (const auto& instance : _instances) {
            instances_json.push_back(*instance);
        }
        json_file << instances_json;
        json_file.flush();

        const auto from = json_new;
        const auto to = json_new.replace_extension();
        fs::rename(from, to);
    } catch (const std::exception& ex) {
        return {-1, ex.what()};
    }

    return {0, {}};
}

} // namespace FLECS
