// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef A26C3D22_DE7E_4EF2_BA44_CB50A45E8C9B
#define A26C3D22_DE7E_4EF2_BA44_CB50A45E8C9B

#include <memory>
#include <set>
#include <string>

#include "app/app.h"
#include "db/app_db.h"
#include "deployment/deployment_docker.h"
#include "module_base/module.h"
#include "util/fs/fs.h"

namespace FLECS {

class module_app_manager_t;
struct instance_config_t;

namespace Private {

class module_app_manager_private_t
{
public:
    module_app_manager_private_t();

    ~module_app_manager_private_t();

    /*! @brief Initializes the module. Sanitizes the app database and starts all previously running app instances
     *
     * @param None
     *
     * @return None
     */
    auto do_init() //
        -> void;

    auto do_load(fs::path base_path = "/var/lib/flecs/") //
        -> void;

    auto do_save(fs::path base_path = "/var/lib/flecs/") const //
        -> void;

    /*! @brief Installs an app from its name and version, i.e. downloads it from the marketplace
     *
     * Downloads the according app manifest and forwards to manifest installation
     *
     * @param[in] app_name Name of the app to install
     * @param[in] version Version of the app to install
     * @param[in] license_key License key to activate with the marketplace
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return Any error code returned by @sa download_manifest
     * @return Any error code returned by overloaded @sa do_install(const std::string&)
     */
    auto do_install(
        const std::string& app_name,
        const std::string& version,
        const std::string& license_key,
        json_t& response) //
        -> crow::status;

    /*! @brief Installs an app from its YAML manifest
     *
     * @param[in] manifest string containing the raw YAML manifest
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_YAML: Error parsing manifest
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     */
    auto do_install(const fs::path& manifest, const std::string& license_key, json_t& response) //
        -> crow::status;

    /*! @brief Sideloads an app from its YAML manifest
     *
     * Copies the transferred app manifest and forwards to manifest installation
     *
     * @param[in] manifest_path Path to a YAML manifest file
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_IOR: Error reading from manifest
     * @return FLECS_IOW: Error writing manifest to FLECS application directory
     * @return Any error code returned by overloaded @sa do_install(const std::string&, const std::string&)
     */
    auto do_sideload(const std::string& yaml, const std::string& license_key, json_t& response) //
        -> crow::status;

    /*! @brief Sideloads an app from its YAML manifest
     *
     * Copies the transferred app manifest and forwards to manifest installation
     *
     * @param[in] manifest_path Path to a YAML manifest file
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_IOR: Error reading from manifest
     * @return FLECS_IOW: Error writing manifest to FLECS application directory
     * @return Any error code returned by overloaded @sa do_install(const std::string&, const std::string&)
     */
    auto do_sideload(const fs::path& manifest_path, const std::string& license_key, json_t& response) //
        -> crow::status;

    /*! @brief Uninstalls an application
     *
     * @param[in] app_name App to uninstall
     * @param[in] version Version to uninstall
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_APP_NOTINST: App not installed
     * @return FLECS_YAML: Error parsing manifest of installed app
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     * @return FLECS_IOW: Error deleting manifest from disk
     */
    auto do_uninstall(const std::string& app_name, const std::string& version, json_t& response, bool force = false) //
        -> crow::status;

    /*! @brief Exports an application
     *
     * @param[in] app_name App to export
     * @param[in] version Version to export
     *
     */
    auto do_export_app(const std::string& app_name, const std::string& version) //
        -> crow::response;

    /*! @brief Creates a new instance of an installed app
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     * @param[in] description Optional: descriptive name of the new instance
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_APP_NOTINST: App not installed in the requested version
     * @return FLECS_YAML: Error parsing manifest of installed app
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     */
    auto do_create_instance(
        const std::string& app_name,
        const std::string& version,
        const std::string& instance_name,
        json_t& response) //
        -> crow::status;

    auto do_update_instance(
        const std::string& instance_id,
        const std::string& app_name,
        const std::string& from,
        const std::string& to,
        json_t& response) //
        -> crow::status;

    /*! @brief Deletes an existing instance
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     * @param[in] app_name Name of the app the instance belongs to or empty string
     * @param[in] version Version of the app the instance belongs to or empty string
     *
     * @return error code
     * @return FLECS_OK No error occurred
     * @return FLECS_INSTANCE_NOTEXIST if the specified instance does not exist
     * @return any error returned by @sa xcheck_app_instance if app_name and/or versions are provided
     */
    auto do_delete_instance(
        const std::string& instance_id,
        const std::string& app_name,
        const std::string& version,
        json_t& response) //
        -> crow::status;

    /*! @brief Starts an existing instance. If the instance is already running, no action is performed and the function
     * call is considered successful. app_name and version can be provided as additional arguments, in which case these
     * values are cross-checked to the contents of the app db.
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     * @param[in] app_name Name of the app the instance belongs to or empty string
     * @param[in] version Version of the app the instance belongs to or empty string
     *
     * @return error code
     * @return FLECS_OK No error occurred
     * @return FLECS_INSTANCE_NOTEXIST if the specified instance does not exist
     * @return FLECS_INSTANCE_NOTRUNNABLE if the specified instance was not successfully created
     * @return FLECS_YAML if the corresponding app manifest is missing or incorrect
     * @return FLECS_DOCKER if the call to Docker was unsuccessful
     * @return any error returned by @sa xcheck_app_instance if app_name and/or versions are provided
     */
    auto do_start_instance(
        const std::string& instance_id,
        const std::string& app_name,
        const std::string& version,
        json_t& response,
        bool internal = false) //
        -> crow::status;

    /*! @brief Stops a running instance. If the instance is not running, no action is performed and the function call is
     * considered successful. app_name and version can be provided as additional arguments, in which case these
     * values are cross-checked to the contents of the app db.
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     * @param[in] app_name Name of the app the instance belongs to or empty string
     * @param[in] version Version of the app the instance belongs to or empty string
     *
     * @return error code
     * @return FLECS_OK No error occurred
     * @return FLECS_INSTANCE_NOTEXIST if the specified instance does not exist
     * @return FLECS_DOCKER if the call to Docker was unsuccessful
     */
    auto do_stop_instance(
        const std::string& instance_id,
        const std::string& app_name,
        const std::string& version,
        json_t& response,
        bool internal = false) //
        -> crow::status;

    /*! @brief Exports an instance
     *
     * @param[in] instance_id instance to export
     *
     */
    auto do_export_instance(const std::string& instance_id) //
        -> crow::response;

    /*! @brief Returns details of an app instance, such as IP address, hostname or exposed ports
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return error code
     */
    auto do_instance_details(const std::string& instance_id, json_t& response) //
        -> crow::status;

    /*! @brief Returns logfile of an app instance
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return error code
     */
    auto do_instance_log(const std::string& instance_id, json_t& response) //
        -> crow::status;

    /*! @brief Prints all installed apps and their instances in JSON format
     *
     * @param None
     *
     * @return FLECS_OK
     */
    auto do_list_apps(json_t& response) //
        -> crow::status;

    /*! @brief Prints all available versions for a given app. Not yet implemented
     */
    auto do_list_versions(const std::string& app_name, json_t& response) //
        -> crow::status;

    /*! @brief Prints all available instances for a given app and version. Not yet implemented
     */
    auto do_list_instances(const std::string& app_name, const std::string& version, json_t& response) //
        -> crow::status;

    auto do_get_config_instance(const std::string& instance_id, json_t& response) //
        -> crow::status;

    auto do_put_config_instance(const std::string& instance_id, const instance_config_t& config, json_t& response) //
        -> crow::status;

private:
    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    auto is_app_installed(const std::string& app_name, const std::string& version) //
        -> bool;

    auto app_versions(std::string_view app_name) const //
        -> std::vector<std::string>;

    /*! @brief Helper function to perform some cross-checks between an instance and a given app_name and version. For
     * some functions, app_name and versions are optional, but if provided these checks will be performed. Used
     * especially for actions triggered through the WebApp to ensure user actions are consistently packed into requests.
     *
     * @param[in] instance Database entry of the instance to check
     * @param[in] app_name Name of the corresponding app
     * @param[in] version Version of the corresponding app
     *
     * @return 0 if check is ok, -1 otherwise
     */
    auto xcheck_app_instance(
        const instance_t& instance,
        const std::string& app_name,
        const std::string& version) //
        -> int;

    auto persist_apps(fs::path base_path = "/var/lib/flecs/apps/") const //
        -> void;
    auto load_apps(fs::path base_path = "/var/lib/flecs/apps/") //
        -> void;

    using installed_apps_t = std::map<app_key_t, app_t, std::less<>>;

    installed_apps_t _installed_apps;
    std::unique_ptr<deployment_t> _deployment;
};

auto build_manifest_path(const std::string& app_name, const std::string& version) //
    -> fs::path;
auto build_manifest_url(const std::string& app_name, const std::string& version) //
    -> std::string;
auto download_manifest(const std::string& app_name, const std::string& version) //
    -> int;

} // namespace Private
} // namespace FLECS

#endif // A26C3D22_DE7E_4EF2_BA44_CB50A45E8C9B
