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

#include "service/app_status.h"

#include "util/sqlite3_ext/sqlite3_db.h"

#include <cstdint>
#include <vector>

namespace FLECS {

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
constexpr const char* apps_table_primary_where_format="app='%s' AND version='%s';";

struct instances_table_primary_t
{
    std::string id;
};
struct instances_table_entry_t: instances_table_primary_t
{
    std::string app;
    std::string version;
    std::string description;
    std::int32_t flags;
};
constexpr const char* instances_table_name = "instances";
constexpr const char* instances_table_primary_where_format="id='%s';";

class app_db_t : protected sqlite3_db_t
{
public:
    app_db_t();

    int insert_app(const apps_table_entry_t& app_entry);

    int delete_app(const apps_table_primary_t& apps_table_primary);

    int insert_instance(const instances_table_entry_t& instance_entry);

    int delete_instance(const instances_table_primary_t& instaces_table_primary);

    apps_table_entry_t query_app(const apps_table_primary_t& primary);

    instances_table_entry_t query_instance(const instances_table_primary_t& primary);

    std::vector<instances_table_entry_t> query_instances(const apps_table_primary_t& instance_entry);

private:
};

} // namespace FLECS

#endif // FLECS_service_app_db_h
