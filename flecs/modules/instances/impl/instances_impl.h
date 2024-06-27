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
#include <mutex>
#include <vector>

#include "flecs/modules/instances/instances.h"

namespace flecs {
namespace jobs {
class progress_t;
} // namespace jobs

namespace deployments {
class deployment_t;
}

namespace module {

class apps_t;
class jobs_t;

namespace impl {

class instances_t
{
    friend class flecs::module::instances_t;

public:
    ~instances_t();

private:
    explicit instances_t(flecs::module::instances_t* parent);

    auto do_load(const fs::path& base_path) //
        -> result_t;

    auto do_module_init() //
        -> void;

    auto do_module_start() //
        -> void;

    auto do_module_stop() //
        -> void;

    auto do_instance_ids(const apps::key_t& app_key) const //
        -> std::vector<instances::id_t>;

    auto do_query(instances::id_t instance_id) const //
        -> std::shared_ptr<instances::instance_t>;

    auto do_is_running(std::shared_ptr<instances::instance_t> instance) const //
        -> bool;

    auto do_list(const apps::key_t& app_key) const //
        -> std::vector<instances::id_t>;

    auto queue_create(apps::key_t app_key, std::string instance_name, bool running) //
        -> jobs::id_t;
    auto do_create_sync(apps::key_t app_key, std::string instance_name, bool running) //
        -> result_t;
    auto do_create(
        apps::key_t app_key, std::string instance_name, bool running, jobs::progress_t& progress) //
        -> result_t;

    auto queue_start(instances::id_t instance_id, bool once) //
        -> jobs::id_t;
    auto do_start_sync(instances::id_t instance_id, bool once) //
        -> result_t;
    auto do_start(instances::id_t instance_id, bool once, jobs::progress_t& progress) //
        -> result_t;

    auto queue_stop(instances::id_t instance_id, bool once) //
        -> jobs::id_t;
    auto do_stop_sync(instances::id_t instance_id, bool once) //
        -> result_t;
    auto do_stop(instances::id_t instance_id, bool once, jobs::progress_t& progress) //
        -> result_t;

    auto queue_remove(instances::id_t instance_id) //
        -> jobs::id_t;
    auto do_remove_sync(instances::id_t instance_id) //
        -> result_t;
    auto do_remove(instances::id_t instance_id, jobs::progress_t& progress) //
        -> result_t;

    auto do_get_config(instances::id_t instance_id) const //
        -> crow::response;

    auto do_post_config(instances::id_t instance_id, const instances::config_t& config) //
        -> crow::response;

    auto do_details(instances::id_t instance_id) const //
        -> crow::response;

    auto do_logs(instances::id_t instance_id) const //
        -> crow::response;

    auto do_get_env(instances::id_t instance_id) const //
        -> crow::response;

    auto do_put_env(instances::id_t instance_id, std::vector<mapped_env_var_t> env_vars) //
        -> crow::response;

    auto do_delete_env(instances::id_t instance_id) //
        -> crow::response;

    auto do_get_ports(instances::id_t instance_id) const //
        -> crow::response;

    auto do_put_ports(instances::id_t instance_id, std::vector<mapped_port_range_t> ports) //
        -> crow::response;

    auto do_delete_ports(instances::id_t instance_id) //
        -> crow::response;

    auto do_get_editor(const crow::request& req, instances::id_t instance_id, uint16_t port) //
        -> crow::response;

    auto queue_update(instances::id_t instance_id, std::string to) //
        -> jobs::id_t;
    auto do_update_sync(instances::id_t instance_id, std::string to) //
        -> result_t;
    auto do_update(instances::id_t instance_id, std::string to, jobs::progress_t& progress) //
        -> result_t;

    auto queue_export_to(instances::id_t instance_id, fs::path base_path) //
        -> jobs::id_t;
    auto do_export_to_sync(instances::id_t instance_id, fs::path base_path) //
        -> result_t;
    auto do_export_to(instances::id_t instance_id, fs::path base_path, jobs::progress_t& progress) //
        -> result_t;

    auto queue_import_from(instances::instance_t instance, fs::path base_path) //
        -> jobs::id_t;
    auto do_import_from_sync(instances::instance_t instance, fs::path base_path) //
        -> result_t;
    auto do_import_from(instances::instance_t instance, fs::path base_path, jobs::progress_t& progress) //
        -> result_t;

    flecs::module::instances_t* _parent;

    std::unique_ptr<deployments::deployment_t> _deployment;
    std::shared_ptr<flecs::module::apps_t> _apps_api;
    std::shared_ptr<flecs::module::jobs_t> _jobs_api;
};

} // namespace impl
} // namespace module
} // namespace flecs
