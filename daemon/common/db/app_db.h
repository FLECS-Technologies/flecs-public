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

#ifndef A73466A9_55BD_49A9_AE77_408A1A82751C
#define A73466A9_55BD_49A9_AE77_408A1A82751C

#include <cstdint>
#include <map>
#include <optional>
#include <vector>

#include "app/app.h"
#include "instance/instance_status.h"
#include "util/sqlite3_ext/sqlite3_db.h"

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
    std::string license_key;
    std::string download_token;
};
struct apps_table_entry_t : public apps_table_primary_t, public apps_table_data_t
{
};
struct apps_table_primary_comparator_t
{
    bool operator()(const apps_table_primary_t& lhs, const apps_table_primary_t& rhs) const
    {
        if (lhs.app < rhs.app) {
            return true;
        }
        if (lhs.app > rhs.app) {
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
    std::vector<std::string> networks;
    std::vector<std::string> ips;
    std::uint32_t flags;
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

    explicit app_db_t(std::string path);

    auto is_open() const noexcept //
        -> bool
    {
        return _ok;
    }

    auto close() //
        -> int
    {
        return sqlite3_db_t::close();
    }

    auto path() const noexcept //
        -> const std::string&
    {
        return _path;
    }

    /*! @brief Returns all apps in the database
     *
     *  @return std::vector containing all apps; empty if none are in the database
     */
    auto all_apps() //
        -> std::vector<apps_table_entry_t>;

    /*! @brief Returns all instances in the app database
     */
    auto all_instances() //
        -> std::vector<instances_table_entry_t>;

private:
    std::string _path;
};

} // namespace FLECS

#endif // A73466A9_55BD_49A9_AE77_408A1A82751C
