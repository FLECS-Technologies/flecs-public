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

#include "module_base/module.h"
#include "util/fs/fs.h"

namespace FLECS {

namespace impl {
class module_apps_t;
} // namespace impl

class app_t;
class app_key_t;
class app_manifest_t;

class module_apps_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
public:
    ~module_apps_t() override;

    /*! @brief Loads installed apps from apps.json
     *
     * @param[in] base_path Path to search for apps.json
     *
     * @return HTTP response
     */
    auto load(fs::path base_path = "/var/lib/flecs/") //
        -> crow::response;

    /*! @brief Save installed apps to apps.json
     *
     * @param[in] base_path Path to store apps.json
     *
     * @return HTTP response
     */
    auto save(fs::path base_path = "/var/lib/flecs/") const //
        -> crow::response;

    /*! @brief Lists all installed Apps
     *
     * @param app_key (optional) limit list to all or specific version of specific App
     *
     * @return HTTP response
     */
    auto list(const app_key_t& app_key) const //
        -> crow::response;
    auto list(std::string app_name, std::string version) const //
        -> crow::response;
    auto list(std::string app_name) const //
        -> crow::response;
    auto list() const //
        -> crow::response;

    /*! @brief Installs an App from the FLECS marketplace
     *
     * @param[in] app_key Key of the App to install, version required @sa app_key_t
     * @param[in] license_key License key to activate with the marketplace
     *
     * @return HTTP response
     */
    auto install_from_marketplace(app_key_t app_key, std::string license_key) //
        -> crow::response;
    auto install_from_marketplace(
        std::string app_name, std::string version, std::string license_key) //
        -> crow::response;
    auto install_from_marketplace(std::string app_name, std::string version) //
        -> crow::response;

    /*! @brief Sideloads an App from its manifest
     *
     * @param[in] manifest_string A valid App manifest as string
     * @param[in] license_key License key to activate with the marketplace
     *
     * @return HTTP response
     */
    auto sideload(std::string manifest_string, std::string license_key) //
        -> crow::response;
    auto sideload(std::string manifest_string) //
        -> crow::response;

    /*! @brief Uninstalls an App
     *
     * @param[in] app_key Key of the App to uninstall, all or specific version @sa app_key_t
     *
     * @return HTTP response
     */
    auto uninstall(app_key_t app_key, bool force) //
        -> crow::response;
    auto uninstall(std::string app_name, std::string version, bool force) //
        -> crow::response;

    /*! @brief Exports an App as compressed archive
     *
     * @param[in] app_key Key of the App to export, all or specific version @sa app_key_t
     *
     */
    auto archive(app_key_t app_key) const //
        -> crow::response;
    auto archive(std::string app_name, std::string version) const //
        -> crow::response;
    auto archive(std::string app_name) const //
        -> crow::response;

    auto contains(const app_key_t& app_key) const noexcept //
        -> bool;
    auto query(const app_key_t& app_key) const noexcept //
        -> std::shared_ptr<app_t>;
    auto is_installed(const app_key_t& app_key) const noexcept //
        -> bool;

protected:
    friend class module_factory_t;

    module_apps_t();

    void do_init() override;
    void do_deinit() override {}

    std::unique_ptr<impl::module_apps_t> _impl;
};

} // namespace FLECS
