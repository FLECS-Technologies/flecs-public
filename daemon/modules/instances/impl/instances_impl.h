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

#include "daemon/modules/instances/instances.h"

namespace flecs {
namespace jobs {

class progress_t;

} // namespace jobs

class deployment_t;

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
        -> std::vector<instance_id_t>;

    auto do_query(instance_id_t instance_id) const //
        -> std::shared_ptr<instance_t>;

    auto do_is_running(std::shared_ptr<instance_t> instance) const //
        -> bool;

    auto do_list(const apps::key_t& app_key) const //
        -> std::vector<instance_id_t>;

    auto queue_create(apps::key_t app_key, std::string instance_name, bool running) //
        -> jobs::id_t;
    auto do_create_sync(apps::key_t app_key, std::string instance_name, bool running) //
        -> result_t;
    auto do_create(
        apps::key_t app_key, std::string instance_name, bool running, jobs::progress_t& progress) //
        -> result_t;

    auto queue_start(instance_id_t instance_id, bool once) //
        -> jobs::id_t;
    auto do_start_sync(instance_id_t instance_id, bool once) //
        -> result_t;
    auto do_start(instance_id_t instance_id, bool once, jobs::progress_t& progress) //
        -> result_t;

    auto queue_stop(instance_id_t instance_id, bool once) //
        -> jobs::id_t;
    auto do_stop_sync(instance_id_t instance_id, bool once) //
        -> result_t;
    auto do_stop(instance_id_t instance_id, bool once, jobs::progress_t& progress) //
        -> result_t;

    auto queue_remove(instance_id_t instance_id) //
        -> jobs::id_t;
    auto do_remove_sync(instance_id_t instance_id) //
        -> result_t;
    auto do_remove(instance_id_t instance_id, jobs::progress_t& progress) //
        -> result_t;

    auto do_get_config(instance_id_t instance_id) const //
        -> crow::response;

    auto do_post_config(instance_id_t instance_id, const instance_config_t& config) //
        -> crow::response;

    auto do_details(instance_id_t instance_id) const //
        -> crow::response;

    auto do_logs(instance_id_t instance_id) const //
        -> crow::response;

    auto queue_update(instance_id_t instance_id, std::string to) //
        -> jobs::id_t;
    auto do_update_sync(instance_id_t instance_id, std::string to) //
        -> result_t;
    auto do_update(instance_id_t instance_id, std::string to, jobs::progress_t& progress) //
        -> result_t;

    auto queue_export_to(instance_id_t instance_id, fs::path base_path) //
        -> jobs::id_t;
    auto do_export_to_sync(instance_id_t instance_id, fs::path base_path) //
        -> result_t;
    auto do_export_to(instance_id_t instance_id, fs::path base_path, jobs::progress_t& progress) //
        -> result_t;

    auto queue_import_from(instance_t instance, fs::path base_path) //
        -> jobs::id_t;
    auto do_import_from_sync(instance_t instance, fs::path base_path) //
        -> result_t;
    auto do_import_from(instance_t instance, fs::path base_path, jobs::progress_t& progress) //
        -> result_t;

    flecs::module::instances_t* _parent;

    std::unique_ptr<deployment_t> _deployment;
    std::shared_ptr<flecs::module::apps_t> _apps_api;
    std::shared_ptr<flecs::module::jobs_t> _jobs_api;
};

} // namespace impl
} // namespace module
} // namespace flecs
