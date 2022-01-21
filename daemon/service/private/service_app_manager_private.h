// Copyright 2021 FLECS Technologies GmbH
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

#ifndef FLECS_service_service_app_manager_private_h
#define FLECS_service_service_app_manager_private_h

#include <string>

#include "db/app_db.h"
#include "service/service_errors.h"

namespace FLECS {

class service_app_manager_t;

namespace Private {

class service_app_manager_private_t
{
public:
    service_app_manager_private_t();
    ~service_app_manager_private_t();

    /*! @brief Installs an application from its name and version, i.e. downloads it from the marketplace
     *
     * @param[in] app_name Name of the application to install
     * @param[in] version Version of the application to install
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_SQLITE: Database error
     * @return Any error code returned by @sa download_manifest
     * @return Any error code returned by overloaded @sa do_install(const std::string&, const std::string&)
     */
    service_error_e do_install(const std::string& app_name, const std::string& version);

    /*! @brief Installs an app from its YAML manifest
     *
     * @param[in] manifest string containing the raw YAML manifest
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_YAML: Error parsing manifest
     * @return FLECS_SQLITE: Database error
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     */
    service_error_e do_install(const std::string& manifest);

    /*! @brief Sideloads an app from its YAML manifest
     *
     * @param[in] manifest_path Path to a YAML manifest file
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_SQLITE: Database error
     * @return FLECS_IOR: Error reading from manifest
     * @return FLECS_IOW: Error writing manifest to FLECS application directory
     * @return Any error code returned by overloaded @sa do_install(const std::string&, const std::string&)
     */
    service_error_e do_sideload(const std::string& manifest_path);

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
     * @return FLECS_SQLITE: Database error
     */
    service_error_e do_uninstall(const std::string& app_name, const std::string& version);

    /*! Creates a new instance of an installed app
     *
     * @param[in] app_name Name of the app
     * @param[in] version Version of the app
     * @param[in] description Optional: descriptive name of the new instance
     *
     * @return error code
     * @return FLECS_OK: No error occurred
     * @return FLECS_APP_NOTINST: App not installed in the requested version
     * @return FLECS_YAML: Error parsing manifest of installed app
     * @return FLECS_SQLITE: Database error
     * @return FLECS_DOCKER: Unsuccessful exit code from spawned Docker process
     */
    service_error_e do_create_instance(
        const std::string& app_name, const std::string& version, const std::string& description);
    service_error_e do_delete_instance(const std::string& app_name, const std::string& version, const std::string& id);
    service_error_e do_start_instance(const std::string& id, const std::string& app, const std::string& version);
    service_error_e do_stop_instance(const std::string& app_name, const std::string& version, const std::string& id);
    service_error_e do_list_apps(const std::string& app_name);
    service_error_e do_list_versions(const std::string& app_name);
    service_error_e do_list_instances(const std::string& app_name, const std::string& version);

    bool is_app_installed(const std::string& app_name, const std::string& version);
    bool is_instance_available(const std::string& app_name, const std::string& version, const std::string& id);
    bool is_instance_runnable(const std::string& app_name, const std::string& version, const std::string& id);
    bool is_instance_running(const std::string& app_name, const std::string& version, const std::string& id);

private:
    app_db_t _app_db;
};

} // namespace Private
} // namespace FLECS

#endif // FLECS_service_service_app_manager_private_h
