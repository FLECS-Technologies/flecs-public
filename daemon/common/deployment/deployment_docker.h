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

#ifndef F9F4DFEC_B3F8_4D33_B5B6_A7B2084181BD
#define F9F4DFEC_B3F8_4D33_B5B6_A7B2084181BD

#include <memory>

#include "deployment.h"

namespace FLECS {

class deployment_docker_t : public deployment_t
{
public:
    deployment_docker_t() = default;

    ~deployment_docker_t() override = default;

private:
    auto do_deployment_id() const noexcept //
        -> std::string_view override;

    auto do_insert_instance(instance_t instance) //
        -> result_t override;
    auto do_create_instance(const app_t& app, instance_t& instance) //
        -> result_t override;
    auto do_delete_instance(std::string_view instance_id) //
        -> result_t override;
    auto do_start_instance(instance_t& instance) //
        -> result_t override;
    auto do_ready_instance(const instance_t& instance) //
        -> result_t override;
    auto do_stop_instance(const instance_t& instance) //
        -> result_t override;
    auto do_is_instance_running(const instance_t& instance) const //
        -> bool override;
    auto do_create_network(
        network_type_t network_type,
        std::string_view network,
        std::string_view cidr_subnet,
        std::string_view gateway,
        std::string_view parent_adapter) //
        -> result_t override;
    auto do_query_network(std::string_view network) //
        -> std::optional<network_t> override;
    auto do_delete_network(std::string_view network) //
        -> result_t override;
    auto do_connect_network(
        std::string_view instance_id,
        std::string_view network,
        std::string_view ip) //
        -> result_t override;
    auto do_disconnect_network(std::string_view instance_id, std::string_view network) //
        -> result_t override;
    auto do_create_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t override;
    auto do_export_volume(const instance_t& instance, std::string_view volume_name, fs::path dest_dir) //
        -> result_t override;
    auto do_delete_volume(std::string_view instance_id, std::string_view volume_name) //
        -> result_t override;
    auto do_copy_file_from_image(std::string_view image, fs::path file, fs::path dest) //
        -> result_t override;
    auto do_copy_file_to_instance(std::string_view instance_id, fs::path file, fs::path dest) //
        -> result_t override;
    auto do_copy_file_from_instance(std::string_view instance_id, fs::path file, fs::path dest) //
        -> result_t override;
    auto do_default_network_name() const //
        -> std::string_view override;
    auto do_default_network_type() const //
        -> network_type_t override;
    auto do_default_network_cidr_subnet() const //
        -> std::string_view override;
    auto do_default_network_gateway() const //
        -> std::string_view override;

    auto create_container(instance_t& instance) //
        -> result_t;
    auto delete_container(const instance_t& instance) //
        -> result_t;
};

} // namespace FLECS

#endif // F9F4DFEC_B3F8_4D33_B5B6_A7B2084181BD
