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

#include "db/app_db.h"

#include <algorithm>
#include <cstring>
#include <iostream>
#include <memory>
#include <sstream>

namespace FLECS {

constexpr const char* const app_db_path = "/var/lib/flecs/db/apps.db";

static int select_apps_callback(void* data, int argc, char** argv, char* col_name[])
{
    auto entries = reinterpret_cast<std::vector<apps_table_entry_t>*>(data);
    decltype(auto) entry = entries->emplace_back();
    for (auto i = 0; i < argc; ++i)
    {
        if (argv[i] == nullptr)
        {
            continue;
        }

        if (strcmp(col_name[i], "app") == 0)
        {
            entry.app = argv[i];
        } else if (strcmp(col_name[i], "version") == 0)
        {
            entry.version = argv[i];
        } else if (strcmp(col_name[i], "status") == 0)
        {
            entry.status = static_cast<app_status_e>(*argv[i]);
        } else if (strcmp(col_name[i], "desired") == 0)
        {
            entry.desired = static_cast<app_status_e>(*argv[i]);
        } else if (strcmp(col_name[i], "category") == 0)
        {
            entry.category = argv[i];
        } else if (strcmp(col_name[i], "installed_size") == 0)
        {
            entry.installed_size = atoi(argv[i]);
        }
    }
    return 0;
}

static int select_instances_callback(void* data, int argc, char** argv, char* col_name[])
{
    auto entries = reinterpret_cast<std::vector<instances_table_entry_t>*>(data);
    decltype(auto) entry = entries->emplace_back();
    for (auto i = 0; i < argc; ++i)
    {
        if (argv[i] == nullptr)
        {
            continue;
        }

        if (strcmp(col_name[i], "id") == 0)
        {
            entry.id = argv[i];
        } else if (strcmp(col_name[i], "app") == 0)
        {
            entry.app = argv[i];
        } else if (strcmp(col_name[i], "version") == 0)
        {
            entry.version = argv[i];
        } else if (strcmp(col_name[i], "status") == 0)
        {
            entry.status = static_cast<instance_status_e>(*argv[i]);
        } else if (strcmp(col_name[i], "desired") == 0)
        {
            entry.desired = static_cast<instance_status_e>(*argv[i]);
        } else if (strcmp(col_name[i], "description") == 0)
        {
            entry.description = argv[i];
        } else if (strcmp(col_name[i], "flags") == 0)
        {
            entry.flags = atoi(argv[i]);
        }
    }
    return 0;
}

app_db_t::app_db_t()
    : sqlite3_db_t{app_db_path, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX, nullptr}
{
    if (_ok)
    {
        constexpr const char* create_apps_table =
            "CREATE TABLE IF NOT EXISTS apps("
            "app TEXT(255),"
            "version TEXT(255),"
            "status CHAR,"
            "desired CHAR,"
            "category INTEGER,"
            "installed_size INTEGER,"
            "PRIMARY KEY(app,version))";

        int res = exec(create_apps_table, nullptr, nullptr);
        if (res != SQLITE_OK)
        {
            close();
        }

        constexpr const char* create_instances_table =
            "CREATE TABLE IF NOT EXISTS instances("
            "id TEXT(255),"
            "app TEXT(255),"
            "version TEXT(255),"
            "status CHAR,"
            "desired CHAR,"
            "description TEXT(4095),"
            "flags INTEGER,"
            "PRIMARY KEY(id))";

        res = exec(create_instances_table, nullptr, nullptr);
        if (res != SQLITE_OK)
        {
            close();
        }
    }
}

int app_db_t::insert_app(const apps_table_entry_t& entry)
{
    const auto sqlite_res = insert_or_replace(
        apps_table_name,
        entry.app,
        entry.version,
        entry.status,
        entry.desired,
        entry.category,
        entry.installed_size);
    if (sqlite_res != SQLITE_OK)
    {
        std::cerr << "Could not insert app into app database: " << sqlite_res << errmsg() << std::endl;
    }
    return sqlite_res;
}

int app_db_t::delete_app(const apps_table_primary_t& primary)
{
    const auto len1 = snprintf(nullptr, 0, delete_statement, apps_table_name);
    const auto len2 =
        snprintf(nullptr, 0, apps_table_primary_where_format, primary.app.c_str(), primary.version.c_str());

    auto delete_str = std::make_unique<char[]>(len1 + len2 + 1);

    std::snprintf(delete_str.get(), len1 + 1, delete_statement, apps_table_name);
    std::snprintf(
        delete_str.get() + len1,
        len2 + 1,
        apps_table_primary_where_format,
        primary.app.c_str(),
        primary.version.c_str());

    return exec(delete_str.get(), nullptr, nullptr);
}

int app_db_t::insert_instance(const instances_table_entry_t& entry)
{
    const auto sqlite_res = insert_or_replace(
        instances_table_name,
        entry.id,
        entry.app,
        entry.version,
        entry.status,
        entry.desired,
        entry.description,
        entry.flags);
    if (sqlite_res != SQLITE_OK)
    {
        std::cerr << "Could not insert instance into app database: " << sqlite_res << errmsg() << std::endl;
    }
    return sqlite_res;
}

int app_db_t::delete_instance(const instances_table_primary_t& primary)
{
    const auto len1 = snprintf(nullptr, 0, delete_statement, instances_table_name);
    const auto len2 = snprintf(nullptr, 0, instances_table_primary_where_format, primary.id.c_str());

    auto delete_str = std::make_unique<char[]>(len1 + len2 + 1);

    std::snprintf(delete_str.get(), len1 + 1, delete_statement, instances_table_name);
    std::snprintf(delete_str.get() + len1, len2 + 1, instances_table_primary_where_format, primary.id.c_str());

    return exec(delete_str.get(), nullptr, nullptr);
}

apps_table_entry_t app_db_t::query_app(const apps_table_primary_t& primary)
{
    auto entries = std::vector<apps_table_entry_t>{};

    std::array<const char*, 2> filter = {"app", "version"};
    select_all(apps_table_name, &select_apps_callback, &entries, filter, primary.app.c_str(), primary.version.c_str());

    return entries.empty() ? apps_table_entry_t{} : entries.front();
}

std::vector<apps_table_entry_t> app_db_t::query_apps()
{
    auto entries = std::vector<apps_table_entry_t>{};

    select_all(apps_table_name, &select_apps_callback, &entries);

    return entries;
}

instances_table_entry_t app_db_t::query_instance(const instances_table_primary_t& primary)
{
    auto entries = std::vector<instances_table_entry_t>{};

    std::array<const char*, 1> filter = {"id"};
    select_all(instances_table_name, &select_instances_callback, &entries, filter, primary.id.c_str());

    return entries.empty() ? instances_table_entry_t{} : entries.front();
}

std::vector<instances_table_entry_t> app_db_t::query_instances(const apps_table_primary_t& instance_entry)
{
    auto entries = std::vector<instances_table_entry_t>{};

    std::array<const char*, 2> filter = {"app", "version"};
    select_all(
        instances_table_name,
        &select_instances_callback,
        reinterpret_cast<void*>(&entries),
        filter,
        instance_entry.app.c_str(),
        instance_entry.version.c_str());

    return entries;
}

} // namespace FLECS
