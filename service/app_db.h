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
#include <vector>

#include "service/app_status.h"
#include "service/instance_status.h"
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
struct apps_table_entry_t : apps_table_primary_t
{
    app_status_e status;
    app_status_e desired;
    std::string category;
    std::int32_t installed_size;

    static constexpr std::size_t MAX_APP_LEN = 255;
};
constexpr const char* apps_table_name = "apps";
constexpr const char* apps_table_primary_where_format = "app='%s' AND version='%s';";

struct instances_table_primary_t
{
    std::string id;
};
struct instances_table_entry_t : instances_table_primary_t
{
    std::string app;
    std::string version;
    std::string description;
    instance_status_e status;
    instance_status_e desired;
    std::int32_t flags;
};
constexpr const char* instances_table_name = "instances";
constexpr const char* instances_table_primary_where_format = "id='%s';";

class app_db_t : public sqlite3_db_t
{
public:
    app_db_t();

    int insert_app(const apps_table_entry_t& app_entry);

    int delete_app(const apps_table_primary_t& apps_table_primary);

    int insert_instance(const instances_table_entry_t& instance_entry);

    int delete_instance(const instances_table_primary_t& instaces_table_primary);

    apps_table_entry_t query_app(const apps_table_primary_t& primary);

    std::vector<apps_table_entry_t> query_apps();

    instances_table_entry_t query_instance(const instances_table_primary_t& primary);

    std::vector<instances_table_entry_t> query_instances(const apps_table_primary_t& instance_entry);

    const char* errmsg() const noexcept { return static_cast<const sqlite3_db_t*>(this)->errmsg(); }

private:
};

} // namespace FLECS

#endif // FLECS_service_app_db_h
