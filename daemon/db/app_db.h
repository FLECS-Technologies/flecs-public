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

#ifndef FLECS_service_app_db_h
#define FLECS_service_app_db_h

#include <cstdint>
#include <map>
#include <optional>
#include <vector>

#include "app/app_status.h"
#include "instance/instance_status.h"
#include "util/sqlite3_ext/sqlite3_db.h"

#ifndef FLECS_APP_DB_PATH
#define FLECS_APP_DB_PATH "/var/lib/flecs/db/apps.db"
#endif // FLECS_APP_DB_PATH

namespace FLECS {

using t = std::remove_cv_t<std::remove_reference_t<const instance_status_e&>>;
constexpr auto is_enum = std::is_enum_v<t>;

constexpr const char* delete_statement = "DELETE FROM %s WHERE ";

struct apps_table_primary_t
{
    std::string app;
    std::string version;
};
struct apps_table_data_t
{
    app_status_e status;
    app_status_e desired;
    std::string category;
    std::int32_t installed_size;
};
struct apps_table_entry_t : public apps_table_primary_t, public apps_table_data_t
{
};
struct apps_table_primary_comparator_t
{
    bool operator()(const apps_table_primary_t& lhs, const apps_table_primary_t& rhs) const
    {
        if (lhs.app < rhs.app)
        {
            return true;
        }
        if (lhs.app > rhs.app)
        {
            return false;
        }
        return lhs.version < rhs.version;
    }
};
constexpr const char* apps_table_name = "apps";
constexpr const char* apps_table_primary_where_format = "app='%s' AND version='%s';";

struct instances_table_primary_t
{
    std::string id;
};
struct instances_table_data_t
{
    std::string app;
    std::string version;
    std::string description;
    instance_status_e status;
    instance_status_e desired;
    std::int32_t flags;
};
struct instances_table_entry_t : public instances_table_primary_t, public instances_table_data_t
{
};
struct instances_table_primary_comparator_t
{
    bool operator()(const instances_table_primary_t& lhs, const instances_table_primary_t& rhs) const
    {
        return lhs.id < rhs.id;
    }
};
constexpr const char* instances_table_name = "instances";
constexpr const char* instances_table_primary_where_format = "id='%s';";

class app_db_t : public sqlite3_db_t
{
public:
    app_db_t();

    ~app_db_t() override;

    int create_app_table();
    int create_instances_table();

    /*! @brief Inserts an app into the app database
     *
     * @param[in] entry
     *
     * @return error code
     */
    void insert_app(const apps_table_entry_t& entry);

    /*! @brief Deletes an app from the app database
     *
     * @param[in] primary
     *
     * @return error code
     */
    void delete_app(const apps_table_primary_t& primary);

    /*! @brief Queries if an app is in the database
     *
     * @param[in] primary
     *
     * @return error code
     */
    bool has_app(const apps_table_primary_t& primary) const noexcept;

    /*! @brief Returns all apps in the database
     *
     *  @return std::vector containing all apps; empty if none are in the database
     */
    std::vector<apps_table_entry_t> all_apps() const;

    /*! @brief Inserts an instance of an app into the app database
     *
     * @param[in] entry
     *
     * @return error code
     */
    void insert_instance(const instances_table_entry_t& entry);

    /*! @brief Deletes an instance of an app from the app database
     *
     * @param[in] primary
     */
    void delete_instance(const instances_table_primary_t& primary);

    /*! @brief Queries if an instance with a given ID is in the app database
     */
    bool has_instance(const instances_table_primary_t& primary) const noexcept;

    /*! @brief Returns all instances in the app database
     */
    std::vector<instances_table_entry_t> all_instances() const;

    /*! @brief Returns all instances for a given app in all versions in the app database
     */
    std::vector<instances_table_entry_t> instances(const std::string& app) const;

    /*! @brief Returns all instances for a given app and version in the app database
     */
    std::vector<instances_table_entry_t> instances(const std::string& app, const std::string& version) const;

    /*! @brief */
    std::optional<apps_table_entry_t> query_app(const apps_table_primary_t& primary) const noexcept;

    /*! @brief */
    std::optional<instances_table_entry_t> query_instance(const instances_table_primary_t& primary) const noexcept;

    int persist();

    const char* errmsg() const noexcept { return static_cast<const sqlite3_db_t*>(this)->errmsg(); }

private:
    void cache_db();

    using apps_table_t = std::map<apps_table_primary_t, apps_table_data_t, apps_table_primary_comparator_t>;
    using instances_table_t =
        std::map<instances_table_primary_t, instances_table_data_t, instances_table_primary_comparator_t>;
    apps_table_t _apps;
    instances_table_t _instances;
};

} // namespace FLECS

#endif // FLECS_service_app_db_h
