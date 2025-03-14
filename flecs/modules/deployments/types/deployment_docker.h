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

#include "deployment.h"

namespace flecs {
namespace deployments {

class docker_t : public deployment_t
{
public:
    docker_t();

    ~docker_t() override;

protected:
    auto docker_login(std::optional<Token> token) const //
        -> result_t;
    auto docker_load(fs::path archive) //
        -> result_t;
    auto docker_save(std::string image, fs::path archive) const //
        -> result_t;
    auto docker_import_volume(std::string volume_name, fs::path src_dir) //
        -> result_t;
    auto docker_export_volume(std::string volume_name, fs::path dest_dir) const //
        -> result_t;

private:
    auto do_deployment_id() const noexcept //
        -> std::string_view override;

    auto do_download_app(std::shared_ptr<apps::app_t> app, std::optional<Token> token) //
        -> result_t override;
    auto do_delete_app(std::shared_ptr<apps::app_t> app) //
        -> result_t override;
    auto do_import_app(std::shared_ptr<apps::app_t> app, fs::path archive) //
        -> result_t override;
    auto do_export_app(std::shared_ptr<const apps::app_t> app, fs::path archive) //
        -> result_t override;
    auto do_determine_app_size(std::shared_ptr<const apps::app_t> app) const //
        -> std::optional<std::size_t> override;

    auto do_create_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
    auto do_delete_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
    auto do_start_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
    auto do_stop_instance(std::shared_ptr<instances::instance_t> instance) //
        -> result_t override;
    auto do_export_instance(std::shared_ptr<instances::instance_t> instance, fs::path dest_dir) const //
        -> result_t override;
    auto do_import_instance(std::shared_ptr<instances::instance_t> instance, fs::path base_dir) //
        -> result_t override;
    auto do_is_instance_running(std::shared_ptr<instances::instance_t> instance) const //
        -> bool override;
    auto do_networks() const -> //
        std::vector<network_t> override;
    auto do_create_network(
        network_type_e network_type,
        std::string network_name,
        std::string cidr_subnet,
        std::string gateway,
        std::string parent_adapter) //
        -> result_t override;
    auto do_query_network(std::string_view network) const //
        -> std::optional<network_t> override;
    auto do_delete_network(std::string_view network) //
        -> result_t override;
    auto do_connect_network(
        std::shared_ptr<instances::instance_t> instance,
        std::string_view network,
        std::string_view ip) //
        -> result_t override;
    auto do_disconnect_network(std::shared_ptr<instances::instance_t> instance, std::string_view network) //
        -> result_t override;
    auto do_create_volume(std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t override;
    auto do_import_volume(
        std::shared_ptr<instances::instance_t> instance,
        volume_t& volume,
        fs::path src_dir) //
        -> result_t override;
    auto do_import_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path src_dir) //
        -> result_t override;
    auto do_export_volume(
        std::shared_ptr<instances::instance_t> instance,
        const volume_t& volume,
        fs::path dest_dir) const //
        -> result_t override;
    auto do_export_volumes(
        std::shared_ptr<instances::instance_t> instance,
        fs::path dest_dir) const //
        -> result_t override;
    auto do_delete_volume(std::shared_ptr<instances::instance_t> instance, std::string_view volume_name) //
        -> result_t override;
    auto do_copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t override;
    auto do_copy_file_to_instance(
        std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) //
        -> result_t override;
    auto do_copy_file_from_instance(
        std::shared_ptr<instances::instance_t> instance, fs::path file, fs::path dest) const //
        -> result_t override;
    auto do_default_network_name() const //
        -> std::string_view override;
    auto do_default_network_type() const //
        -> network_type_e override;
    auto do_default_network_cidr_subnet() const //
        -> std::string_view override;
    auto do_default_network_gateway() const //
        -> std::string_view override;

    auto create_container(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
    auto delete_container(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;
};

} // namespace deployments
} // namespace flecs
