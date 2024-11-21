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

#include "flecs/modules/deployments/types/deployment.h"

#include <algorithm>
#include <fstream>
#include <iostream>
#include <map>
#include <regex>
#include <set>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#ifdef FLECS_MOCK_MODULES
#include "flecs/modules/floxy/__mocks__/floxy.h"
#else // FLECS_MOCK_MODULES
#include "flecs/modules/floxy/floxy.h"
#endif // FLECS_MOCK_MODULES
#include "flecs/modules/factory/factory.h"
#include "flecs/util/network/ip_addr.h"
#include "flecs/util/network/network.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace deployments {

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

auto deployment_t::download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
    -> result_t
{
    return do_download_app(std::move(app), std::move(token));
}

auto deployment_t::delete_app(std::shared_ptr<apps::app_t> app) //
    -> result_t
{
    return do_delete_app(std::move(app));
}

auto deployment_t::import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
    -> result_t
{
    return do_import_app(std::move(app), std::move(archive));
}

auto deployment_t::export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
    -> result_t
{
    return do_export_app(std::move(app), std::move(archive));
}

auto deployment_t::determine_app_size(std::shared_ptr<const apps::app_t> app) const //
    -> std::optional<std::size_t>
{
    return do_determine_app_size(std::move(app));
}

auto deployment_t::instance_ids() const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{});
}

auto deployment_t::instance_ids(std::string_view app) const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{app.data(), ""});
}

auto deployment_t::instance_ids(std::string_view app, std::string_view version) const //
    -> std::vector<instances::id_t>
{
    return instance_ids(apps::key_t{app.data(), version.data()});
}

auto deployment_t::instance_ids(const apps::key_t& app_key) const //
    -> std::vector<instances::id_t>
{
    auto ids = std::vector<instances::id_t>{};
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

auto deployment_t::query_instance(instances::id_t instance_id) const //
    -> std::shared_ptr<instances::instance_t>
{
    const auto it = std::find_if(
        _instances.cbegin(),
        _instances.cend(),
        [&instance_id](const std::shared_ptr<instances::instance_t>& elem) {
            return elem->id() == instance_id;
        });
    return it != _instances.cend() ? *it : nullptr;
}

auto deployment_t::has_instance(instances::id_t instance_id) const noexcept //
    -> bool
{
    const auto it = std::find_if(
        _instances.cbegin(),
        _instances.cend(),
        [&instance_id](const std::shared_ptr<instances::instance_t>& elem) {
            return elem->id() == instance_id;
        });
    return it != _instances.cend();
}

auto deployment_t::insert_instance(instances::instance_t instance) //
    -> std::shared_ptr<instances::instance_t>
{
    return _instances.emplace_back(std::make_shared<instances::instance_t>(std::move(instance)));
}

auto deployment_t::create_instance(std::shared_ptr<const apps::app_t> app, std::string instance_name) //
    -> result_t
{
    auto manifest = app->manifest();
    if (!manifest) {
        return {-1, "Could not access app manifest"};
    }

    // Step 1: Create instance and generate unique id
    auto tmp = instances::instance_t{app, instance_name};
    while (has_instance(tmp.id())) {
        tmp.regenerate_id();
    }

    // Step 2: Create port mapping that does not conflict with existing port mappings
    // Conflicting host ports will be replaced to let docker choose random free host ports
    auto ports = std::vector<mapped_port_range_t>{};
    constexpr auto port_range_for_random_host = port_range_t{port_t{0}, port_t{0}};
    for (auto& port : manifest->ports()) {
        if (do_host_ports_collide(port.host_port_range())) {
            ports.emplace_back(port_range_for_random_host, port.container_port_range());
        } else {
            ports.emplace_back(port);
        }
    }

    // Step 3: Insert instance
    tmp.status(instances::status_e::Requested);
    tmp.desired(instances::status_e::Created);

    auto instance = insert_instance(std::move(tmp));
    for (const auto& startup_option : manifest->startup_options()) {
        instance->startup_options().emplace_back(
            static_cast<std::underlying_type_t<startup_option_t>>(startup_option));
    }

    // Step 4: Add environment variables and port mappings
    instance->set_environment(manifest->env());
    instance->set_ports(std::move(ports));

    // Step 5: Create volumes
    {
        auto [res, additional_info] = create_volumes(instance);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    // Step 6: Create networks
    // query and create default network, if required
    const auto network_name = default_network_name();
    if (!network_name.empty()) {
        const auto network = query_network(network_name);
        if (!network.has_value()) {
            const auto [res, additional_info] = create_network(
                default_network_type(),
                std::string{default_network_name()},
                std::string{default_network_cidr_subnet()},
                std::string{default_network_gateway()},
                {});
            if (res != 0) {
                return {-1, instance->id().hex()};
            }
        }
        auto mac_address =
            manifest->networks().empty() ? std::string{} : manifest->networks()[0].mac_address();
        instance->networks().emplace_back(instances::instance_t::network_t{
            .network_name = default_network_name().data(),
            .mac_address = std::move(mac_address),
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

    // Step 7: Create conffiles
    {
        auto [res, additional_info] = create_config_files(instance);
        if (res != 0) {
            return {res, instance->id().hex()};
        }
        instance->status(instances::status_e::ResourcesReady);
    }

    return do_create_instance(instance);
}

auto deployment_t::delete_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    const auto [res, additional_info] = do_delete_instance(instance);
    const auto floxy_api = dynamic_cast<module::floxy_t*>(api::query_module("floxy").get());
    floxy_api->delete_reverse_proxy_configs(instance);
    _instances.erase(
        std::remove_if(
            _instances.begin(),
            _instances.end(),
            [&instance](const std::shared_ptr<instances::instance_t>& elem) {
                return elem->id() == instance->id();
            }),
        _instances.end());
    return {res, additional_info};
}

auto deployment_t::start_instance(std::shared_ptr<instances::instance_t> instance) //
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

    // Create config for reverse proxy
    std::optional<std::string> instance_ip;
    auto app = instance->app();
    for (const auto& network : instance->networks()) {
        if (network.network_name == "flecs") {
            instance_ip = network.ip_address;
            break;
        }
    }
    if (instance_ip.has_value()) {
        auto editor_ports = std::vector<uint16_t>{};
        for (const auto& [_, editor] : app->manifest()->editors()) {
            if (editor.supports_reverse_proxy()) {
                editor_ports.push_back(editor.port());
            }
        }
        if (!editor_ports.empty()) {
            const auto floxy_api = dynamic_cast<module::floxy_t*>(api::query_module("floxy").get());
            auto [ec, message] = floxy_api->load_instance_reverse_proxy_config(
                instance_ip.value(),
                app->key().name(),
                instance->id(),
                editor_ports);
            if (ec != 0) {
                std::cerr << "Loading reverse proxy config for " + instance->instance_name() + " failed: "
                          << message << std::endl;
            }
        }
    }

    return ready_instance(std::move(instance));
}

auto deployment_t::ready_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    return do_ready_instance(std::move(instance));
}

auto deployment_t::stop_instance(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto [res, additional_info] = do_stop_instance(instance);
    const auto floxy_api = dynamic_cast<module::floxy_t*>(api::query_module("floxy").get());
    floxy_api->delete_server_proxy_configs(instance);
    instance->clear_editor_port_mapping();
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

auto deployment_t::export_instance(
    std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    LOG_TRACE("--> %s Request to export instance %s\n", __FUNCTION__, instance->id().hex().c_str());

    auto app = instance->app();
    if (!app) {
        LOG_TRACE("<-- %s Instance not connected to an app\n", __FUNCTION__);
        return {-1, "Instance not connected to an app"};
    }

    auto manifest = app->manifest();
    if (!manifest) {
        LOG_TRACE("<-- %s Could not access app manifest\n", __FUNCTION__);
        return {-1, "Could not access app manifest"};
    }

    dest_dir /= instance->id().hex();
    auto ec = std::error_code{};
    fs::create_directories(dest_dir, ec);
    if (ec) {
        LOG_TRACE("<-- %s Could not create export directory %s\n", __FUNCTION__, dest_dir.c_str());
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

    std::tie(res, additional_info) = do_export_instance(instance, dest_dir);
    LOG_TRACE("<-- %s %s\n", __FUNCTION__, additional_info.c_str());

    return {res, additional_info};
}

auto deployment_t::import_instance(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
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

    for (auto& network : instance->networks()) {
        auto net = query_network(network.network_name);
        if (!net.has_value()) {
            return {-1, "Could not find network " + network.network_name};
        }
        auto result_address = transfer_ip_to_network(net.value(), network.ip_address);
        if (result_address.has_value()) {
            network.ip_address = to_string(result_address.value());
        } else {
            return {-1, "Could not transfer ip " + network.ip_address + " to network " + net.value().name};
        }
    }

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

auto deployment_t::is_instance_runnable(std::shared_ptr<instances::instance_t> instance) const //
    -> bool
{
    return instance && instance->status() == instances::status_e::Created;
}

auto deployment_t::is_instance_running(std::shared_ptr<instances::instance_t> instance) const //
    -> bool
{
    return instance && do_is_instance_running(std::move(instance));
}

auto deployment_t::do_host_ports_collide(const port_range_t& port_range) const //
    -> bool
{
    for (auto instance : _instances) {
        auto ports = instance->ports();
        if (ports.has_value()) {
            for (auto& existing_port_range : ports.value()) {
                if (port_range.does_collide_with(existing_port_range.host_port_range())) {
                    return true;
                }
            }
        }
    }
    return false;
}

auto deployment_t::create_config_files(std::shared_ptr<instances::instance_t> instance) //
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

        const auto [res, additional_info] =
            copy_file_from_image(manifest->image_with_tag(), conffile.container(), local_path);
        if (res != 0) {
            return {-1, instance->id().hex()};
        }
    }

    return {0, {}};
}

auto deployment_t::networks() const //
    -> std::vector<network_t>
{
    return do_networks();
}

auto deployment_t::create_network(
    network_type_e network_type,
    std::string network_name,
    std::string cidr_subnet,
    std::string gateway,
    std::string parent_adapter) //
    -> result_t
{
    return do_create_network(
        std::move(network_type),
        std::move(network_name),
        std::move(cidr_subnet),
        std::move(gateway),
        std::move(parent_adapter));
}

auto deployment_t::query_network(std::string_view network) const //
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
    std::shared_ptr<instances::instance_t> instance,
    std::string_view network,
    std::string_view ip) //
    -> result_t
{
    return do_connect_network(std::move(instance), std::move(network), std::move(ip));
}

auto deployment_t::disconnect_network(
    std::shared_ptr<instances::instance_t> instance, std::string_view network) //
    -> result_t
{
    return do_disconnect_network(std::move(instance), std::move(network));
}

auto deployment_t::create_volumes(std::shared_ptr<instances::instance_t> instance) //
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
    std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
    -> result_t
{
    return do_create_volume(std::move(instance), std::move(volume_name));
}

auto deployment_t::import_volumes(std::shared_ptr<instances::instance_t> instance, fs::path src_dir) //
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
            const auto [res, additional_info] = import_volume(instance, volume, src_dir);
            if (res != 0) {
                return {res, additional_info};
            }
        }
    }

    return {0, {}};
}

auto deployment_t::import_volume(
    std::shared_ptr<instances::instance_t> instance, volume_t& volume, fs::path src_dir) //
    -> result_t
{
    if (volume.type() != volume_t::VOLUME) {
        return {-1, "Cannot import non-volume " + volume.host()};
    }

    auto ec = std::error_code{};
    if (!fs::exists(src_dir, ec) || !fs::is_directory(src_dir, ec)) {
        return {-1, "Source directory does not exist"};
    }

    return do_import_volume(std::move(instance), volume, src_dir);
}

auto deployment_t::export_volumes(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    LOG_TRACE(
        "--> %s Request to export volumes of instance %s to %s\n",
        __FUNCTION__,
        instance->id().hex().c_str(),
        dest_dir.c_str());

    auto app = instance->app();
    if (!app) {
        LOG_TRACE("<-- %s Instance not connected to an app\n", __FUNCTION__);
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        LOG_TRACE("<-- %s Could not access app manifest\n", __FUNCTION__);
        return {-1, "Could not access app manifest"};
    }

    for (auto& volume : manifest->volumes()) {
        if (volume.type() == volume_t::VOLUME) {
            const auto [res, additional_info] = export_volume(instance, volume, dest_dir);
            if (res != 0) {
                return {res, additional_info};
            }
        }
    }

    return {0, {}};
}

auto deployment_t::export_volume(
    std::shared_ptr<instances::instance_t> instance,
    const volume_t& volume,
    fs::path dest_dir) const //
    -> result_t
{
    LOG_TRACE(
        "--> %s Request to export volume %s of instance %s to %s\n",
        __FUNCTION__,
        volume.host().c_str(),
        instance->id().hex().c_str(),
        dest_dir.c_str());

    if (volume.type() != volume_t::VOLUME) {
        LOG_TRACE("<-- %s Cannot export non-volume %s\n", __FUNCTION__, volume.host().c_str());
        return {-1, "Cannot export non-volume " + volume.host()};
    }

    auto ec = std::error_code{};
    fs::create_directories(dest_dir, ec);
    if (ec) {
        LOG_TRACE("<-- %s Could not create export directory\n", __FUNCTION__);
        return {-1, "Could not create export directory"};
    }

    auto [res, message] = do_export_volume(std::move(instance), volume, std::move(dest_dir));

    LOG_TRACE("<-- %s %s\n", __FUNCTION__, message.c_str());
    return {res, message};
}

auto deployment_t::export_config_files(
    std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
    -> result_t
{
    LOG_TRACE(
        "--> %s Request to export config files of instance %s to %s\n",
        __FUNCTION__,
        instance->id().hex().c_str(),
        dest_dir.c_str());

    auto app = instance->app();
    if (!app) {
        LOG_TRACE("<-- %s Instance not connected to an app\n", __FUNCTION__);
        return {-1, "Instance not connected to an app"};
    }
    auto manifest = app->manifest();
    if (!manifest) {
        LOG_TRACE("<-- %s Could not access app manifest\n", __FUNCTION__);
        return {-1, "Could not access app manifest"};
    }

    for (auto& config_file : manifest->conffiles()) {
        const auto [res, additional_info] = export_config_file(instance, config_file, dest_dir);
        if (res != 0) {
            return {res, additional_info};
        }
    }

    LOG_TRACE("<-- %s\n", __FUNCTION__);
    return {0, {}};
}
auto deployment_t::export_config_file(
    std::shared_ptr<instances::instance_t> instance,
    const conffile_t& config_file,
    fs::path dest_dir) const //
    -> result_t
{
    LOG_TRACE(
        "--> %s Request to export config file %s of instance %s to %s\n",
        __FUNCTION__,
        config_file.container().c_str(),
        instance->id().hex().c_str(),
        dest_dir.c_str());

    auto ec = std::error_code{};
    fs::create_directories(dest_dir, ec);
    if (ec) {
        LOG_TRACE("<-- %s Could not create export directory\n", __FUNCTION__);
        return {-1, "Could not create export directory"};
    }

    if (is_instance_running(instance)) {
        LOG_TRACE("--- %s Exporting config file from running instance\n", __FUNCTION__);
        const auto [res, additional_info] =
            copy_file_from_instance(instance, config_file.container(), dest_dir / config_file.local());
        if (res != 0) {
            return {res, additional_info};
        }
    } else {
        LOG_TRACE("--- %s Exporting config file from local directory\n", __FUNCTION__);
        const auto conf_path = "/var/lib/flecs/instances/" + instance->id().hex() + "/conf/";
        /* copy config files from local dir for stopped instances */
        auto ec = std::error_code{};
        fs::copy(conf_path + config_file.local(), dest_dir / config_file.local(), ec);
        if (ec) {
            LOG_TRACE("<-- %s Could not export conffile from local directory\n", __FUNCTION__);
            return {-1, "Could not export conffile from local directory"};
        }
    }

    LOG_TRACE("<-- %s\n", __FUNCTION__);
    return {0, {}};
}

auto deployment_t::import_config_files(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
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
    std::shared_ptr<instances::instance_t> instance, const conffile_t& config_file, fs::path base_dir) //
    -> result_t
{
    const auto conf_dir = fs::path{"/var/lib/flecs/instances/"} / instance->id().hex() / "conf";
    /* copy config files from local dir for stopped instances */
    auto ec = std::error_code{};
    fs::create_directories(conf_dir, ec);
    fs::copy(
        base_dir / config_file.local(),
        conf_dir / config_file.local(),
        fs::copy_options::overwrite_existing,
        ec);
    if (ec) {
        return {-1, "Could not import conffile"};
    }

    return {0, {}};
}

auto deployment_t::delete_volumes(std::shared_ptr<instances::instance_t> instance) //
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
    std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
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
    std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) //
    -> result_t
{
    return do_copy_file_to_instance(std::move(instance), file, dest);
}

auto deployment_t::copy_file_from_instance(
    std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) const //
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

auto deployment_t::transfer_ip_to_network(const network_t& network, std::string_view ip_address) const //
    -> std::optional<ip_addr_t>
{
    auto base_ip = get_base_ip(network.cidr_subnet);
    if (!base_ip.has_value()) {
        return {};
    }
    auto net_size = get_subnet_size(network.cidr_subnet);
    if (!net_size.has_value()) {
        return {};
    }
    auto num = ip_addr_t{ip_address}.addr_v4();
    // Remove network part from ip
    num.s_addr &= ~0u << net_size.value();
    // Combine with new network base
    num.s_addr |= base_ip.value().addr_v4().s_addr;
    return ip_addr_t{num};
}

auto deployment_t::get_base_ip(std::string_view cidr_subnet) const //
    -> std::optional<ip_addr_t>
{
    // parse a.b.c.d
    // from beginning of line: (a.b.c.)(d/)
    // e.g. 127.0.0.1/24 -> (127.0.0.)(1)/
    const auto ip_regex = std::regex{R"((^(?:\d{1,3}\.){3}\d{1,3})\/)"};
    auto m = std::cmatch{};
    if (!std::regex_search(cidr_subnet.data(), m, ip_regex)) {
        return {};
    }
    return ip_addr_t{m[1].str()};
}

auto deployment_t::get_subnet_size(std::string_view cidr_subnet) const //
    -> std::optional<int>
{
    // parse /x
    // until end of line: d/[0-32]
    // e.g. 127.0.0.1/24 -> 1/24
    const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
    auto m = std::cmatch{};
    if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2) {
        return {};
    }
    return std::stoi(m[1].str());
}

auto deployment_t::generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
    -> std::string
{
    auto base_ip = get_base_ip(cidr_subnet);
    if (!base_ip.has_value()) {
        return {};
    }

    auto subnet_size = get_subnet_size(cidr_subnet);
    if (!subnet_size.has_value()) {
        return {};
    }

    // determine the last ip address that belongs to the subnet:
    // (~0u << subnet_size) creates an inverted bitmask (such as 0xff00),
    // which is again inverted (yielding e.g. 0x00ff) and or'ed with the base ip.
    // finally subtract 1 to exclude the network's broadcast address
    const auto max_ip = ip_addr_t{(base_ip.value().addr_v4().s_addr | ~(~0u << subnet_size.value())) - 1};

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
    auto instance_ip = base_ip.value() + 2;

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
            _instances.push_back(
                std::make_shared<instances::instance_t>(instance.get<instances::instance_t>()));
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

} // namespace deployments
} // namespace flecs
