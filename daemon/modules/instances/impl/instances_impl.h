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

#include "instances.h"

namespace FLECS {

class deployment_t;
class job_progress_t;
class module_apps_t;
class module_jobs_t;

namespace impl {

class module_instances_t
{
    friend class FLECS::module_instances_t;

public:
    ~module_instances_t();

private:
    explicit module_instances_t(FLECS::module_instances_t* parent);

    auto do_load(const fs::path& base_path) //
        -> void;

    auto do_init() //
        -> void;

    auto do_instance_ids(const app_key_t& app_key) const //
        -> std::vector<instance_id_t>;

    auto do_query(instance_id_t instance_id) const //
        -> std::shared_ptr<instance_t>;

    auto do_is_running(std::shared_ptr<instance_t> instance) const //
        -> bool;

    auto do_list(const app_key_t& app_key) const //
        -> std::vector<instance_id_t>;

    auto queue_create(app_key_t app_key, std::string instance_name) //
        -> job_id_t;
    auto do_create_sync(app_key_t app_key, std::string instance_name) //
        -> result_t;
    auto do_create(app_key_t app_key, std::string instance_name, job_progress_t& progress) //
        -> result_t;

    auto queue_start(instance_id_t instance_id, bool once) //
        -> job_id_t;
    auto do_start_sync(instance_id_t instance_id, bool once) //
        -> result_t;
    auto do_start(instance_id_t instance_id, bool once, job_progress_t& progress) //
        -> result_t;

    auto queue_stop(instance_id_t instance_id, bool once) //
        -> job_id_t;
    auto do_stop_sync(instance_id_t instance_id, bool once) //
        -> result_t;
    auto do_stop(instance_id_t instance_id, bool once, job_progress_t& progress) //
        -> result_t;

    auto queue_remove(instance_id_t instance_id) //
        -> job_id_t;
    auto do_remove_sync(instance_id_t instance_id) //
        -> result_t;
    auto do_remove(instance_id_t instance_id, job_progress_t& progress) //
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
        -> job_id_t;
    auto do_update_sync(instance_id_t instance_id, std::string to) //
        -> result_t;
    auto do_update(instance_id_t instance_id, std::string to, job_progress_t& progress) //
        -> result_t;

    auto queue_export_to(instance_id_t instance_id, fs::path base_path) //
        -> job_id_t;
    auto do_export_to_sync(instance_id_t instance_id, fs::path base_path) //
        -> result_t;
    auto do_export_to(instance_id_t instance_id, fs::path base_path, job_progress_t& progress) //
        -> result_t;

    auto queue_import_from(instance_id_t instance_id, fs::path base_path) //
        -> job_id_t;
    auto do_import_from_sync(instance_id_t instance_id, fs::path base_path) //
        -> result_t;
    auto do_import_from(instance_id_t instance_id, fs::path base_path, job_progress_t& progress) //
        -> result_t;

    FLECS::module_instances_t* _parent;

    std::unique_ptr<deployment_t> _deployment;
    std::shared_ptr<FLECS::module_apps_t> _apps_api;
    std::shared_ptr<FLECS::module_jobs_t> _jobs_api;
};

} // namespace impl
} // namespace FLECS
