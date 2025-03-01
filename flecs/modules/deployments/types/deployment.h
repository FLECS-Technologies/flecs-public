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

#include <map>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"
#include "flecs/common/network/network_type.h"
#include "flecs/core/flecs.h"
#include "flecs/modules/instances/types/instance.h"
#include "flecs/util/fs/fs.h"
#include "flecs/util/network/ip_addr.h"

namespace flecs {
namespace apps {
class app_t;
class key_t;
} // namespace apps

class conffile_t;
class volume_t;

namespace deployments {

class deployment_t
{
public:
    struct network_t
    {
        std::string name;
        std::string cidr_subnet;
        std::string gateway;
        std::string parent;
        network_type_e type;
    };

    deployment_t() = default;

    virtual ~deployment_t() = default;

    auto deployment_id() const noexcept //
        -> std::string_view;

    auto load(const fs::path& base_path = "/var/lib/flecs/") //
        -> result_t;
    auto save(const fs::path& base_path = "/var/lib/flecs/") //
        -> result_t;

    auto download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
        -> result_t;
    auto delete_app(std::shared_ptr<apps::app_t> app) //
        -> result_t;
    auto import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
        -> result_t;
    auto export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
        -> result_t;
    auto determine_app_size(std::shared_ptr<const apps::app_t> app) const //
        -> std::optional<std::size_t>;

    auto instance_ids(const apps::key_t& app_key) const //
        -> std::vector<instances::id_t>;
    auto instance_ids(std::string_view app, std::string_view version) const //
        -> std::vector<instances::id_t>;
    auto instance_ids(std::string_view app) const //
        -> std::vector<instances::id_t>;
    auto instance_ids() const //
        -> std::vector<instances::id_t>;
    auto query_instance(instances::id_t instance_id) const //
        -> std::shared_ptr<instances::instance_t>;
    auto has_instance(instances::id_t instance_id) const noexcept //
        -> bool;
    auto insert_instance(instances::instance_t instance) //
        -> std::shared_ptr<instances::instance_t>;
    auto create_instance(std::shared_ptr<const apps::app_t> app, std::string instance_name) //
        -> result_t;
    auto delete_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto start_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto stop_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto export_instance(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
        -> result_t;
    auto import_instance(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
        -> result_t;
    auto is_instance_runnable(std::shared_ptr<instances::instance_t> instance) const //
        -> bool;
    auto is_instance_running(std::shared_ptr<instances::instance_t> instance) const //
        -> bool;
    auto do_host_ports_collide(const port_range_t& port_range) const //
        -> bool;
    auto create_config_files(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto create_network(
        network_type_e network_type,
        std::string network_name,
        std::string cidr_subnet,
        std::string gateway,
        std::string parent_adapter) //
        -> result_t;
    auto networks() const //
        -> std::vector<network_t>;
    auto query_network(std::string_view network) const //
        -> std::optional<network_t>;
    auto delete_network(std::string_view network) //
        -> result_t;
    auto connect_network(
        std::shared_ptr<instances::instance_t> instance,
        std::string_view network,
        std::string_view ip) //
        -> result_t;
    auto disconnect_network(std::shared_ptr<instances::instance_t>, std::string_view network) //
        -> result_t;
    auto create_volumes(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto create_volume(std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t;
    auto import_volumes(std::shared_ptr<instances::instance_t> instance, fs::path src_dir) //
        -> result_t;
    auto import_volume(
        std::shared_ptr<instances::instance_t> instance,
        volume_t& volume,
        fs::path src_dir) //
        -> result_t;
    auto export_volumes(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
        -> result_t;
    auto export_volume(
        std::shared_ptr<instances::instance_t> instance,
        const volume_t& volume,
        fs::path dest_dir) const //
        -> result_t;
    auto export_config_files(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
        -> result_t;
    auto export_config_file(
        std::shared_ptr<instances::instance_t> instance,
        const conffile_t& config_file,
        fs::path dest_dir) const //
        -> result_t;
    auto import_config_files(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
        -> result_t;
    auto import_config_file(
        std::shared_ptr<instances::instance_t> instance, const conffile_t& config_file, fs::path base_dir) //
        -> result_t;
    auto delete_volumes(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto delete_volume(std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t;
    auto copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t;
    auto copy_file_to_instance(
        std::shared_ptr<instances::instance_t> instance_id, fs::path file, fs::path dest) //
        -> result_t;
    auto copy_file_from_instance(
        std::shared_ptr<instances::instance_t> instance_id, fs::path file, fs::path dest) const //
        -> result_t;
    auto default_network_name() const //
        -> std::string_view;
    auto default_network_type() const //
        -> network_type_e;
    auto default_network_cidr_subnet() const //
        -> std::string_view;
    auto default_network_gateway() const //
        -> std::string_view;
    auto transfer_ip_to_network(const network_t& network, std::string_view ip_address) const //
        -> std::optional<ip_addr_t>;
    auto get_base_ip(std::string_view cidr_subnet) const //
        -> std::optional<ip_addr_t>;
    auto get_subnet_size(std::string_view cidr_subnet) const //
        -> std::optional<int>;

    auto generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
        -> std::string;

protected:
    std::vector<std::shared_ptr<instances::instance_t>> _instances;
    std::map<std::string, network_t> _networks;

private:
    auto do_load(fs::path json_file_path) //
        -> result_t;

    auto do_save(const fs::path& base_path) //
        -> result_t;

    virtual auto do_deployment_id() const noexcept //
        -> std::string_view = 0;

    virtual auto do_download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
        -> result_t = 0;
    virtual auto do_delete_app(std::shared_ptr<apps::app_t> app) //
        -> result_t = 0;
    virtual auto do_import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
        -> result_t = 0;
    virtual auto do_export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
        -> result_t = 0;
    virtual auto do_determine_app_size(std::shared_ptr<const apps::app_t> app) const //
        -> std::optional<std::size_t> = 0;

    virtual auto do_create_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t = 0;
    virtual auto do_delete_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t = 0;
    virtual auto do_start_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t = 0;
    virtual auto do_stop_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t = 0;
    virtual auto do_export_instance(
        std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
        -> result_t = 0;
    virtual auto do_import_instance(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
        -> result_t = 0;
    virtual auto do_is_instance_running(std::shared_ptr<instances::instance_t> instance) const //
        -> bool = 0;
    virtual auto do_networks() const //
        -> std::vector<network_t> = 0;
    virtual auto do_create_network(
        network_type_e network_type,
        std::string network_name,
        std::string cidr_subnet,
        std::string gateway,
        std::string parent_adapter) //
        -> result_t = 0;
    virtual auto do_query_network(std::string_view network) const //
        -> std::optional<network_t> = 0;
    virtual auto do_delete_network(std::string_view network) //
        -> result_t = 0;
    virtual auto do_connect_network(
        std::shared_ptr<instances::instance_t> instance,
        std::string_view network,
        std::string_view ip) //
        -> result_t = 0;
    virtual auto do_disconnect_network(
        std::shared_ptr<instances::instance_t> instance, std::string_view network) //
        -> result_t = 0;
    virtual auto do_create_volume(
        std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t = 0;
    virtual auto do_import_volume(
        std::shared_ptr<instances::instance_t> instance,
        volume_t& volume,
        fs::path src_dir) //
        -> result_t = 0;
    virtual auto do_import_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path src_dir) //
        -> result_t = 0;
    virtual auto do_export_volume(
        std::shared_ptr<instances::instance_t> instance,
        const volume_t& volume,
        fs::path dest_dir) const //
        -> result_t = 0;
    virtual auto do_export_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path dest_dir) const //
        -> result_t = 0;
    virtual auto do_delete_volume(
        std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t = 0;
    virtual auto do_copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t = 0;
    virtual auto do_copy_file_to_instance(
        std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) //
        -> result_t = 0;
    virtual auto do_copy_file_from_instance(
        std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) const //
        -> result_t = 0;
    virtual auto do_default_network_name() const //
        -> std::string_view = 0;
    virtual auto do_default_network_type() const //
        -> network_type_e = 0;
    virtual auto do_default_network_cidr_subnet() const //
        -> std::string_view = 0;
    virtual auto do_default_network_gateway() const //
        -> std::string_view = 0;
};

} // namespace deployments
} // namespace flecs
