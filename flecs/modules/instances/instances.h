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

#include "flecs/common/app/manifest/variable/variable.h"
#include "flecs/modules/instances/types/instance_id.h"
#include "flecs/modules/jobs/types/job_id.h"
#include "flecs/modules/module_base/module.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace apps {
class key_t;
} // namespace apps

namespace instances {
class config_t;
class instance_t;
} // namespace instances

namespace module {
namespace impl {
class instances_t;
} // namespace impl

class instances_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
public:
    ~instances_t() override;

    /*! @brief Lists all available instances
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return HTTP response
     */
    auto http_list(const apps::key_t& app_key) const //
        -> crow::response;

    auto http_details(instances::id_t instance_id) const //
        -> crow::response;

    auto http_create(apps::key_t app_key, std::string instance_name, bool running) //
        -> crow::response;

    auto http_start(instances::id_t instance_id) //
        -> crow::response;

    auto http_stop(instances::id_t instance_id) //
        -> crow::response;

    auto http_remove(instances::id_t instance_id) //
        -> crow::response;

    auto http_get_config(instances::id_t instance_id) const //
        -> crow::response;

    auto http_post_config(instances::id_t instance_id, const instances::config_t& config) //
        -> crow::response;

    auto http_logs(instances::id_t instance_id) const //
        -> crow::response;

    auto http_update(instances::id_t instance_id, std::string to) //
        -> crow::response;

    auto http_export_to(instances::id_t instance_id, fs::path dest_dir) const //
        -> crow::response;

    auto http_get_env(instances::id_t instance_id) const //
        -> crow::response;

    auto http_put_env(instances::id_t instance_id, std::vector<mapped_env_var_t> env_vars) //
        -> crow::response;

    auto http_delete_env(instances::id_t instance_id) //
        -> crow::response;

    /*! @brief List all available instance ids
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return vector containing all available instance ids
     */
    auto instance_ids(const apps::key_t& app_key) const //
        -> std::vector<instances::id_t>;
    auto instance_ids(std::string app_name, std::string version) const //
        -> std::vector<instances::id_t>;
    auto instance_ids(std::string app_name) const //
        -> std::vector<instances::id_t>;
    auto instance_ids() const //
        -> std::vector<instances::id_t>;

    /*! @brief Query instance for a given instance id
     *
     * @param instance_id instance id to query
     *
     * @return shared_ptr to instance, or nullptr
     */
    auto query(instances::id_t instance_id) const //
        -> std::shared_ptr<instances::instance_t>;

    /*! @brief Query if instance is running
     *
     * @param instance shared_ptr to instance
     *
     * @return true if instance is running, false otherwise
     */
    auto is_running(std::shared_ptr<instances::instance_t> instance) const //
        -> bool;

    auto create(apps::key_t app_key, std::string instance_name, bool running) //
        -> result_t;
    auto create(apps::key_t app_key) //
        -> result_t;
    auto create(std::string app_name, std::string version, std::string instance_name) //
        -> result_t;
    auto create(std::string app_name, std::string version) //
        -> result_t;

    auto start(instances::id_t instance_id) //
        -> result_t;
    auto start_once(instances::id_t instance_id) //
        -> result_t;

    auto stop(instances::id_t instance_id) //
        -> result_t;
    auto stop_once(instances::id_t instance_id) //
        -> result_t;

    auto remove(instances::id_t instance_id) //
        -> result_t;

    auto export_to(instances::id_t instance_id, fs::path base_path) const //
        -> result_t;

    auto import_from(instances::instance_t instance, fs::path base_path) //
        -> result_t;

protected:
    friend class factory_t;

    instances_t();

    auto do_load(const fs::path& base_path) //
        -> result_t override;

    auto do_init() //
        -> void override;

    auto do_start() //
        -> void override;

    auto do_stop() //
        -> void override;

    auto do_deinit() //
        -> void override
    {}

    std::unique_ptr<impl::instances_t> _impl;
};

} // namespace module
} // namespace flecs
