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

#include "apps.h"
#include "common/app/app.h"

namespace FLECS {

class job_progress_t;
class module_instances_t;
class module_manifests_t;
class module_jobs_t;

namespace impl {

class module_apps_t
{
    friend class FLECS::module_apps_t;

public:
    ~module_apps_t();

private:
    explicit module_apps_t(FLECS::module_apps_t* parent);

    auto do_init() //
        -> void;

    auto do_load(fs::path base_path = "/var/lib/flecs/") //
        -> crow::response;

    auto do_save(fs::path base_path = "/var/lib/flecs/") const //
        -> crow::response;

    auto do_list(const app_key_t& app_key) const //
        -> crow::response;

    auto queue_install_from_marketplace(app_key_t app_key, std::string license_key) //
        -> crow::response;

    auto do_install_from_marketplace(
        app_key_t app_key,
        std::string license_key,
        job_progress_t& progress) //
        -> result_t;

    auto queue_sideload(std::string manifest_string, std::string license_key) //
        -> crow::response;

    auto do_sideload(
        std::string manifest_string, std::string license_key, job_progress_t& progress) //
        -> result_t;

    auto queue_uninstall(app_key_t app_key, bool force) //
        -> crow::response;

    auto do_uninstall(app_key_t app_key, bool force, job_progress_t& progress) //
        -> result_t;

    auto queue_archive(app_key_t app_key) const //
        -> crow::response;

    auto do_archive(app_key_t app_key, job_progress_t& progress) const //
        -> result_t;

    auto do_install_impl(
        std::shared_ptr<app_manifest_t> manifest,
        std::string_view license_key,
        job_progress_t& progress) //
        -> result_t;

    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto do_contains(const app_key_t& app_key) const noexcept //
        -> bool;

    auto do_query(const app_key_t& app_key) const noexcept //
        -> std::shared_ptr<app_t>;

    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto do_is_installed(const app_key_t& app_key) const noexcept //
        -> bool;

    FLECS::module_apps_t* _parent;

    std::vector<std::shared_ptr<app_t>> _apps;
    std::mutex _apps_mutex;

    std::shared_ptr<FLECS::module_manifests_t> _manifests_api;
    std::shared_ptr<FLECS::module_jobs_t> _jobs_api;
};

} // namespace impl
} // namespace FLECS
