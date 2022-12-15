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

#include "app/app.h"
#include "core/flecs.h"
#include "instance/instance.h"
#include "util/fs/fs.h"

namespace FLECS {

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

    enum version_filter_e {
        MatchVersion = 0,
        AllVersions = 1,
    };

    deployment_t() = default;

    virtual ~deployment_t() = default;

    auto deployment_id() const noexcept //
        -> std::string_view;

    auto load(fs::path base_path = "/var/lib/flecs/deployment/") //
        -> result_t;
    auto save(fs::path base_path = "/var/lib/flecs/deployment/") //
        -> result_t;

    auto instances() noexcept //
        -> std::map<std::string, instance_t>&;
    auto instances() const noexcept //
        -> const std::map<std::string, instance_t>&;
    auto instance_ids(std::string_view app) const //
        -> std::vector<std::string>;
    auto instance_ids(std::string_view app, std::string_view version) const //
        -> std::vector<std::string>;
    auto instance_ids(const app_key_t& app_key, version_filter_e version_filter = AllVersions) const //
        -> std::vector<std::string>;
    auto instance_ids(const app_t& app, version_filter_e version_filter = AllVersions) const //
        -> std::vector<std::string>;
    auto has_instance(std::string_view instance_id) const noexcept //
        -> bool;
    auto insert_instance(instance_t instance) //
        -> result_t;
    auto create_instance(const app_t& app, std::string instance_name) //
        -> result_t;
    auto delete_instance(std::string_view instance_id) //
        -> result_t;
    auto start_instance(std::string_view instance_id) //
        -> result_t;
    auto ready_instance(std::string_view instance_id) //
        -> result_t;
    auto stop_instance(std::string_view instance_id) //
        -> result_t;
    auto export_instance(const instance_t& instance, fs::path dest_dir) const //
        -> result_t;
    auto is_instance_runnable(std::string_view instance_id) const //
        -> bool;
    auto is_instance_running(std::string_view instance_id) const //
        -> bool;
    auto create_conffiles(const instance_t& instance) //
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
    auto create_volumes(const instance_t& instance) //
        -> result_t;
    auto create_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t;
    auto import_volumes(const instance_t& instance, fs::path src_dir) //
        -> result_t;
    auto import_volume(const instance_t& instance, std::string_view volume_name, fs::path src_dir) //
        -> result_t;
    auto export_volumes(const instance_t& instance, fs::path dest_dir) const //
        -> result_t;
    auto export_volume(const instance_t& instance, std::string_view volume_name, fs::path dest_dir) const //
        -> result_t;
    auto delete_volumes(const instance_t& instance) //
        -> result_t;
    auto delete_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t;
    auto copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t;
    auto copy_file_to_instance(std::string_view instance_id, fs::path file, fs::path dest) //
        -> result_t;
    auto copy_file_from_instance(std::string_view instance_id, fs::path file, fs::path dest) const //
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
    auto do_load(fs::path base_path) //
        -> result_t;

    auto do_save(fs::path base_path) //
        -> result_t;

    virtual auto do_deployment_id() const noexcept //
        -> std::string_view = 0;

    virtual auto do_insert_instance(instance_t instance) //
        -> result_t = 0;
    virtual auto do_create_instance(const app_t& app, instance_t& instance) //
        -> result_t = 0;
    virtual auto do_delete_instance(std::string_view instance_id) //
        -> result_t = 0;
    virtual auto do_start_instance(instance_t& instance) //
        -> result_t = 0;
    virtual auto do_ready_instance(const instance_t& instance) //
        -> result_t = 0;
    virtual auto do_stop_instance(const instance_t& instance) //
        -> result_t = 0;
    virtual auto do_export_instance(const instance_t& instance, fs::path dest_dir) const //
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
    virtual auto do_import_volume(const instance_t& instance, std::string_view volume_name, fs::path src_dir) //
        -> result_t = 0;
    virtual auto do_export_volume(const instance_t& instance, std::string_view volume_name, fs::path dest_dir) const //
        -> result_t = 0;
    virtual auto do_delete_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t = 0;
    virtual auto do_copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t = 0;
    virtual auto do_copy_file_to_instance(std::string_view instance_id, fs::path file, fs::path dest) //
        -> result_t = 0;
    virtual auto do_copy_file_from_instance(std::string_view instance_id, fs::path file, fs::path dest) const //
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

} // namespace FLECS

#endif // BD6EE81F_EC42_4122_8EE7_5036BA499377
