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

#include "common/app/app.h"
#include "module_base/module.h"
#include "util/fs/fs.h"

namespace FLECS {

class job_progress_t;
class module_apps_t;
class module_instances_t;
class module_jobs_t;

namespace impl {

class module_apps_impl_t
{
    friend class FLECS::module_apps_t;

public:
    ~module_apps_impl_t();

private:
    explicit module_apps_impl_t(module_apps_t* parent);

    auto do_init() //
        -> void;

    auto do_load(fs::path base_path = "/var/lib/flecs/") //
        -> crow::response;

    auto do_save(fs::path base_path = "/var/lib/flecs/") const //
        -> crow::response;

    auto do_list(std::string_view app_name, std::string_view version) const //
        -> crow::response;

    auto queue_install_from_marketplace(std::string app_name, std::string version, std::string license_key) //
        -> crow::response;

    auto do_install_from_marketplace(
        std::string app_name,
        std::string version,
        std::string license_key,
        job_progress_t& progress) //
        -> void;

    auto queue_sideload(std::string manifest_string, std::string license_key) //
        -> crow::response;

    auto do_sideload(std::string manifest_string, std::string license_key, job_progress_t& progress) //
        -> void;

    auto queue_uninstall(std::string app_name, std::string version, bool force) //
        -> crow::response;

    auto do_uninstall(std::string app_name, std::string version, bool force, job_progress_t& progress) //
        -> void;

    auto queue_export_app(std::string app_name, std::string version) const //
        -> crow::response;

    auto do_export_app(std::string app_name, std::string version, job_progress_t& progress) const //
        -> void;

    auto do_install_impl(const fs::path& manifest, std::string_view license_key, job_progress_t& progress) //
        -> void;

    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto has_app(std::string_view app_name, std::string_view version) const noexcept //
        -> bool;

    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto is_app_installed(std::string_view app_name, std::string_view version) const noexcept //
        -> bool;

    module_apps_t* _parent;

    using installed_apps_t = std::map<app_key_t, app_t, std::less<>>;
    installed_apps_t _installed_apps;

    std::mutex _installed_apps_mutex;

    std::shared_ptr<FLECS::module_instances_t> _instances_api;
    std::shared_ptr<FLECS::module_jobs_t> _jobs_api;
};

} // namespace impl
} // namespace FLECS
