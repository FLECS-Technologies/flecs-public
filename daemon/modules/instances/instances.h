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
#include <vector>

#include "common/instance/instance_id.h"
#include "module_base/module.h"
#include "modules/jobs/job_id.h"
#include "util/fs/fs.h"

namespace FLECS {

class app_key_t;
class instance_config_t;
class instance_t;

namespace impl {
class module_instances_t;
} // namespace impl

class module_instances_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
public:
    ~module_instances_t() override;

    /*! @brief Lists all available instances
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return HTTP response
     */
    auto http_list(const app_key_t& app_key) const //
        -> crow::response;

    auto http_details(instance_id_t instance_id) const //
        -> crow::response;

    auto http_create(app_key_t app_key, std::string instance_name) //
        -> crow::response;

    auto http_start(instance_id_t instance_id) //
        -> crow::response;

    auto http_stop(instance_id_t instance_id) //
        -> crow::response;

    auto http_remove(instance_id_t instance_id) //
        -> crow::response;

    auto http_get_config(instance_id_t instance_id) const //
        -> crow::response;

    auto http_post_config(instance_id_t instance_id, const instance_config_t& config) //
        -> crow::response;

    auto http_logs(instance_id_t instance_id) const //
        -> crow::response;

    auto http_update(instance_id_t instance_id, std::string to) //
        -> crow::response;

    auto http_export_to(instance_id_t instance_id, fs::path dest_dir) const //
        -> crow::response;

    /*! @brief List all available instance ids
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return vector containing all available instance ids
     */
    auto instance_ids(const app_key_t& app_key) const //
        -> std::vector<instance_id_t>;
    auto instance_ids(std::string app_name, std::string version) const //
        -> std::vector<instance_id_t>;
    auto instance_ids(std::string app_name) const //
        -> std::vector<instance_id_t>;
    auto instance_ids() const //
        -> std::vector<instance_id_t>;

    /*! @brief Query instance for a given instance id
     *
     * @param instance_id instance id to query
     *
     * @return shared_ptr to instance, or nullptr
     */
    auto query(instance_id_t instance_id) const //
        -> std::shared_ptr<instance_t>;

    /*! @brief Query if instance is running
     *
     * @param instance shared_ptr to instance
     *
     * @return true if instance is running, false otherwise
     */
    auto is_running(std::shared_ptr<instance_t> instance) const //
        -> bool;

    auto create(app_key_t app_key, std::string instance_name) //
        -> result_t;
    auto create(app_key_t app_key) //
        -> result_t;
    auto create(std::string app_name, std::string version, std::string instance_name) //
        -> result_t;
    auto create(std::string app_name, std::string version) //
        -> result_t;

    auto start(instance_id_t instance_id) //
        -> result_t;
    auto start_once(instance_id_t instance_id) //
        -> result_t;

    auto stop(instance_id_t instance_id) //
        -> result_t;
    auto stop_once(instance_id_t instance_id) //
        -> result_t;

    auto remove(instance_id_t instance_id) //
        -> result_t;

    auto export_to(instance_id_t instance_id, fs::path base_path) const //
        -> result_t;

    auto import_from(instance_t instance, fs::path base_path) //
        -> result_t;

protected:
    friend class module_factory_t;

    module_instances_t();

    auto do_load(const fs::path& base_path) //
        -> void override;

    auto do_init() //
        -> void override;

    auto do_start() //
        -> void override;

    auto do_deinit() //
        -> void override
    {}

    std::unique_ptr<impl::module_instances_t> _impl;
};

} // namespace FLECS
