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
#include <mutex>
#include <string>

#include "flecs/modules/apps/apps.h"
#include "flecs/modules/apps/types/app.h"

namespace flecs {
namespace jobs {
class progress_t;
} // namespace jobs

namespace module {

class deployments_t;
class instances_t;
class manifests_t;
class jobs_t;

namespace impl {

class apps_t
{
    friend class flecs::module::apps_t;

public:
    ~apps_t();

private:
    explicit apps_t(flecs::module::apps_t* parent);

    auto do_module_init() //
        -> void;

    auto do_load(const fs::path& base_path) //
        -> result_t;

    auto do_module_start() //
        -> void;

    auto do_save(const fs::path& base_path) const //
        -> result_t;

    auto do_app_keys(const apps::key_t& app_key) const //
        -> std::vector<apps::key_t>;

    auto queue_install_from_marketplace(apps::key_t app_key) //
        -> jobs::id_t;
    auto queue_install_many_from_marketplace(std::vector<apps::key_t> app_keys) //
        -> jobs::id_t;
    auto do_install_from_marketplace_sync(apps::key_t app_key) //
        -> result_t;
    auto do_install_many_from_marketplace_sync(std::vector<apps::key_t> app_keys) //
        -> result_t;
    auto do_install_from_marketplace(apps::key_t app_key, jobs::progress_t& progress) //
        -> result_t;
    auto do_install_many_from_marketplace(std::vector<apps::key_t> app_keys, jobs::progress_t& progress) //
        -> result_t;
    auto install_from_marketplace(apps::key_t app_key, jobs::progress_t& progress) //
        -> result_t;

    auto queue_sideload(std::string manifest_string) //
        -> jobs::id_t;
    auto do_sideload_sync(std::string manifest_string) //
        -> result_t;
    auto do_sideload(std::string manifest_string, jobs::progress_t& progress) //
        -> result_t;

    auto queue_uninstall(apps::key_t app_key) //
        -> jobs::id_t;
    auto do_uninstall_sync(apps::key_t app_key) //
        -> result_t;
    auto do_uninstall(apps::key_t app_key, jobs::progress_t& progress) //
        -> result_t;

    auto queue_export_to(apps::key_t app_key, fs::path dest_dir) const //
        -> jobs::id_t;
    auto do_export_to_sync(apps::key_t app_key, fs::path dest_dir) const //
        -> result_t;
    auto do_export_to(apps::key_t app_key, fs::path dest_dir, jobs::progress_t& progress) const //
        -> result_t;

    auto queue_import_from(apps::key_t app_key, fs::path src_dir) //
        -> jobs::id_t;
    auto do_import_from_sync(apps::key_t app_key, fs::path src_dir) //
        -> result_t;
    auto do_import_from(apps::key_t app_key, fs::path src_dir, jobs::progress_t& progress) //
        -> result_t;

    auto do_install_impl(
        std::shared_ptr<app_manifest_t> manifest,
        jobs::progress_t& progress) //
        -> result_t;

    auto do_query(const apps::key_t& app_key) const noexcept //
        -> std::shared_ptr<apps::app_t>;

    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto do_is_installed(const apps::key_t& app_key) const noexcept //
        -> bool;

    flecs::module::apps_t* _parent;

    std::vector<std::shared_ptr<apps::app_t>> _apps;
    std::mutex _apps_mutex;

    std::shared_ptr<flecs::module::deployments_t> _deployments_api;
    std::shared_ptr<flecs::module::instances_t> _instances_api;
    std::shared_ptr<flecs::module::manifests_t> _manifests_api;
    std::shared_ptr<flecs::module::jobs_t> _jobs_api;
};

} // namespace impl
} // namespace module
} // namespace flecs
