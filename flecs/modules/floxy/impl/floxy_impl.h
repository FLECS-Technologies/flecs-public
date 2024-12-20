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

#include <string>

#include "flecs/modules/floxy/floxy.h"
#include "flecs/modules/instances/types/instance_id.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace module {
namespace impl {

class floxy_t
{
    friend class flecs::module::floxy_t;

private:
    floxy_t(flecs::module::floxy_t* parent);

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    auto clear_server_configs(const fs::path base_path = "/var/lib/flecs/") //
        -> void;

    auto build_instance_config_path(
        const std::string& app_name,
        const instances::id_t& instance_id,
        const fs::path base_path = "/var/lib/flecs/") //
        -> fs::path;

    auto build_server_config_path(
        const std::string& app_name,
        const instances::id_t& instance_id,
        uint16_t host_port,
        const fs::path base_path = "/var/lib/flecs/") //
        -> fs::path;

    static auto get_main_config_path() //
        -> fs::path;

    auto reload_floxy_config() //
        -> result_t;

    static auto create_instance_config(
        const instances::id_t& instance_id, const std::string& instance_address, uint16_t dest_port) //
        -> std::string;

    auto create_server_config(const std::string& instance_address, uint16_t host_port, uint16_t dest_port) //
        -> std::string;

    auto load_reverse_proxy_config(const std::string& content, const fs::path& file_path) //
        -> result_t;

    auto do_load_instance_reverse_proxy_config(
        const std::string& ip_address,
        const std::string& app_name,
        const instances::id_t& instance_id,
        std::vector<std::uint16_t>& dest_ports) //
        -> result_t;

    auto delete_reverse_proxy_config(const fs::path& file_path, bool reload = true) //
        -> result_t;

    auto do_delete_reverse_proxy_configs(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;

    auto delete_server_config(
        const std::string& app_name,
        const instances::id_t& instance_id,
        std::uint16_t host_port,
        bool reload = true) //
        -> result_t;

    auto do_delete_server_proxy_configs(
        std::shared_ptr<instances::instance_t> instance, bool reload = true) //
        -> result_t;

    auto do_redirect_editor_request(instances::id_t instance_id, std::uint16_t port) //
        -> crow::response;

    auto redirect_editor_request_to_free_port(
        std::shared_ptr<instances::instance_t> instance, std::uint16_t dest_port) //
        -> crow::response;

    flecs::module::floxy_t* _parent;
};

} // namespace impl
} // namespace module
} // namespace flecs
