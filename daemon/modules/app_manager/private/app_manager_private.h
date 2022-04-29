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

#include <filesystem>
#include <string>

#include "db/app_db.h"
#include "module_base/module.h"
#include "util/http/status_codes.h"

namespace Json {
class Value;
} // namespace Json

namespace FLECS {

class module_app_manager_t;

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
    void do_init();

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
    http_status_e do_install(
        const std::string& app_name, const std::string& version, const std::string& license_key, Json::Value& response);

    /*! @brief Installs an app from its YAML manifest
     *
     * @param[in] manifest string containing the raw YAML manifest
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_YAML: Error parsing manifest
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     */
    http_status_e do_install(const std::string& manifest, const std::string& license_key, Json::Value& response);

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
    http_status_e do_sideload(const std::string& yaml, const std::string& license_key, Json::Value& response);

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
    http_status_e do_sideload(
        const std::filesystem::path& manifest_path, const std::string& license_key, Json::Value& response);

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
    http_status_e do_uninstall(
        const std::string& app_name, const std::string& version, Json::Value& response, bool force = false);

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
    http_status_e do_create_instance(
        const std::string& app_name, const std::string& version, const std::string& description, Json::Value& response);

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
    http_status_e do_delete_instance(
        const std::string& id, const std::string& app_name, const std::string& version, Json::Value& response);

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
    http_status_e do_start_instance(
        const std::string& id, const std::string& app_name, const std::string& version, Json::Value& response,
        bool internal = false);

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
    http_status_e do_stop_instance(
        const std::string& id, const std::string& app_name, const std::string& version, Json::Value& response,
        bool internal = false);

    /*! @brief Returns details of an app instance, such as IP address, hostname or exposed ports
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return error code
     */
    http_status_e do_instance_details(const std::string& id, Json::Value& response);

    /*! @brief Returns logfile of an app instance
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return error code
     */
    http_status_e do_instance_log(const std::string& id, Json::Value& response);

    /*! @brief Prints all installed apps and their instances in JSON format
     *
     * @param None
     *
     * @return FLECS_OK
     */
    http_status_e do_list_apps(Json::Value& response);

    /*! @brief Prints all available versions for a given app. Not yet implemented
     */
    http_status_e do_list_versions(const std::string& app_name, Json::Value& response);

    /*! @brief Prints all available instances for a given app and version. Not yet implemented
     */
    http_status_e do_list_instances(const std::string& app_name, const std::string& version, Json::Value& response);

private:
    /*! @brief Helper function to determine whether a given app is installed in a given version
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     *
     * @return true if the app is installed, false otherwise
     */
    bool is_app_installed(const std::string& app_name, const std::string& version);

    /*! @brief Helper function to determine whether a given instance is runnable, i.e. successfully created
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return true if instance is runnable, false otherwise
     */
    bool is_instance_runnable(const std::string& id);

    /*! @brief Helper function to determine whether a given instance is running. Queries Docker to determine the status.
     *
     * @param[in] id Unique instance id assigned by @sa do_create_instance
     *
     * @return true if instance is running, false otherwise
     */
    bool is_instance_running(const std::string& id);

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
    int xcheck_app_instance(
        const instances_table_entry_t& instance, const std::string& app_name, const std::string& version);

    std::string generate_instance_ip();

    app_db_t _app_db;
};

std::string build_manifest_path(const std::string& app_name, const std::string& version);
std::string build_manifest_url(const std::string& app_name, const std::string& version);
int download_manifest(const std::string& app_name, const std::string& version);

} // namespace Private
} // namespace FLECS

#endif // A26C3D22_DE7E_4EF2_BA44_CB50A45E8C9B
