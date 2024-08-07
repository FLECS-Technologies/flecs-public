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
#include <string>
#include <vector>

#include "flecs/modules/jobs/types/job_id.h"
#include "flecs/modules/module_base/module.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace apps {
class app_t;
class key_t;
} // namespace apps

class app_manifest_t;

namespace module {
namespace impl {
class apps_t;
} // namespace impl

class apps_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
public:
    ~apps_t() override;

    /*! @brief Loads installed apps from apps.json
     *
     * @param[in] base_path Path to search for apps.json
     *
     * @return HTTP response
     */
    auto do_load(const fs::path& base_path) //
        -> result_t override;

    auto do_start() //
        -> void override;

    /*! @brief Save installed apps to apps.json
     *
     * @param[in] base_path Path to store apps.json
     *
     * @return HTTP response
     */
    auto do_save(const fs::path& base_path) const //
        -> result_t override;

    auto http_list(const apps::key_t& app_key) const //
        -> crow::response;

    auto http_install(apps::key_t app_key) //
        -> crow::response;

    auto http_install_many(std::vector<apps::key_t> app_keys) //
        -> crow::response;

    auto http_sideload(std::string manifest_string) //
        -> crow::response;

    auto http_uninstall(apps::key_t app_key) //
        -> crow::response;

    auto http_export_to(apps::key_t) //
        -> crow::response;

    auto app_keys(const apps::key_t& app_key) const //
        -> std::vector<apps::key_t>;
    auto app_keys(std::string app_name, std::string version) const //
        -> std::vector<apps::key_t>;
    auto app_keys(std::string app_name) const //
        -> std::vector<apps::key_t>;
    auto app_keys() const //
        -> std::vector<apps::key_t>;

    auto query(const apps::key_t& app_key) const noexcept //
        -> std::shared_ptr<apps::app_t>;

    /*! @brief Installs an App from the FLECS marketplace
     *
     * @param[in] app_key Key of the App to install, version required @sa apps::key_t
     *
     * @return HTTP response
     */
    auto install_from_marketplace(apps::key_t app_key) //
        -> result_t;

    auto install_many_from_marketplace(std::vector<apps::key_t> app_keys) //
        -> result_t;

    /*! @brief Sideloads an App from its manifest
     *
     * @param[in] manifest_string A valid App manifest as string
     *
     * @return HTTP response
     */
    auto sideload(std::string manifest_string) //
        -> result_t;

    /*! @brief Uninstalls an App
     *
     * @param[in] app_key Key of the App to uninstall, all or specific version @sa apps::key_t
     *
     * @return HTTP response
     */
    auto uninstall(apps::key_t app_key) //
        -> result_t;

    /*! @brief Exports an App as compressed archive
     *
     * @param[in] app_key Key of the App to export, all or specific version @sa apps::key_t
     *
     */
    auto export_to(apps::key_t app_key, fs::path dest_dir) const //
        -> result_t;

    auto import_from(apps::key_t app_key, fs::path src_dir) //
        -> result_t;

    auto is_installed(const apps::key_t& app_key) const noexcept //
        -> bool;

protected:
    friend class factory_t;

    apps_t();

    void do_init() override;
    void do_deinit() override {}

    std::unique_ptr<impl::apps_t> _impl;
};

} // namespace module
} // namespace flecs
