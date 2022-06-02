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

#include "app_db.h"

#include <algorithm>
#include <cstring>
#include <filesystem>
#include <iostream>
#include <memory>
#include <sstream>
#include <string_view>

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

        auto col = std::string_view(col_name[i]);
        if (col == "app")
        {
            entry.app = argv[i];
        }
        else if (col == "version")
        {
            entry.version = argv[i];
        }
        else if (col == "status")
        {
            entry.status = static_cast<app_status_e>(*argv[i]);
        }
        else if (col == "desired")
        {
            entry.desired = static_cast<app_status_e>(*argv[i]);
        }
        else if (col == "category")
        {
            entry.category = argv[i];
        }
        else if (col == "installed_size")
        {
            entry.installed_size = atoi(argv[i]);
        }
        else if (col == "license_key")
        {
            entry.license_key = argv[i];
        }
        else if (col == "download_token")
        {
            entry.download_token = argv[i];
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

        auto col = std::string_view{col_name[i]};
        if (col == "id")
        {
            entry.id = argv[i];
        }
        else if (col == "app")
        {
            entry.app = argv[i];
        }
        else if (col == "version")
        {
            entry.version = argv[i];
        }
        else if (col == "status")
        {
            entry.status = static_cast<instance_status_e>(*argv[i]);
        }
        else if (col == "desired")
        {
            entry.desired = static_cast<instance_status_e>(*argv[i]);
        }
        else if (col == "description")
        {
            entry.description = argv[i];
        }
        else if (col == "networks")
        {
            auto networks = split(argv[i], ',');
            for (decltype(auto) network : networks)
            {
                entry.networks.emplace_back(network);
            }
        }
        else if (col == "ipv4_addr" || col == "ip_addr")
        {
            auto ips = split(argv[i], ',');
            for (decltype(auto) ip : ips)
            {
                entry.ips.emplace_back(ip);
            }
        }
        else if (col == "flags")
        {
            entry.flags = atoi(argv[i]);
        }
    }
    return 0;
}

static int user_version_callback(void* data, int argc, char** argv, char* col_name[])
{
    auto version = reinterpret_cast<int*>(data);
    for (auto i = 0; i < argc; ++i)
    {
        if (argv[i] == nullptr)
        {
            continue;
        }

        if (strcmp(col_name[i], "user_version") == 0)
        {
            *version = std::stoi(argv[i]);
        }
    }
    return 0;
}

app_db_t::app_db_t()
    : app_db_t{app_db_path}
{}

app_db_t::app_db_t(std::string path)
    : sqlite3_db_t{path.c_str(), SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX, nullptr}
    , _apps{}
    , _instances{}
    , _path{path}
{
    if (_ok)
    {
        create_app_table();
        create_instances_table();
    }

    cache_db();
    migrate_db();
}

app_db_t::~app_db_t()
{
    persist();
}

int app_db_t::create_app_table()
{
    return create_table(
        apps_table_name,
        sqlite3_column_t{"app", SQLITE3_TEXT, 255},
        sqlite3_column_t{"version", SQLITE3_TEXT, 255},
        sqlite3_column_t{"status", SQLITE3_TEXT, 1},
        sqlite3_column_t{"desired", SQLITE3_TEXT, 1},
        sqlite3_column_t{"category", SQLITE3_TEXT, 255},
        sqlite3_column_t{"installed_size", SQLITE_INTEGER},
        sqlite3_column_t{"license_key", SQLITE3_TEXT, 255},
        sqlite3_column_t{"download_token", SQLITE3_TEXT, 8192},
        sqlite3_primary_t{"app, version"});
}

int app_db_t::create_instances_table()
{
    return create_table(
        instances_table_name,
        sqlite3_column_t{"id", SQLITE3_TEXT, 255},
        sqlite3_column_t{"app", SQLITE3_TEXT, 255},
        sqlite3_column_t{"version", SQLITE3_TEXT, 255},
        sqlite3_column_t{"status", SQLITE3_TEXT, 1},
        sqlite3_column_t{"desired", SQLITE3_TEXT, 1},
        sqlite3_column_t{"description", SQLITE3_TEXT, 4096},
        sqlite3_column_t{"networks", SQLITE3_TEXT, 4096},
        sqlite3_column_t{"ipv4_addr", SQLITE3_TEXT, 4096},
        sqlite3_column_t{"flags", SQLITE_INTEGER},
        sqlite3_primary_t{"id"});
}

int app_db_t::user_version() const noexcept
{
    return _user_version;
}

int app_db_t::set_user_version()
{
    const auto stmt = std::string{"PRAGMA user_version = "} + std::to_string(CURRENT_USER_VERSION) + std::string{";"};
    return exec(stmt.c_str(), nullptr, nullptr);
}

int app_db_t::query_user_version()
{
    return exec("PRAGMA user_version;", user_version_callback, &_user_version);
}

void app_db_t::insert_app(const app_t& app)
{
    const auto primary = apps_table_primary_t{app.app(), app.version()};
    const auto data = apps_table_data_t{
        app.status(),
        app.desired(),
        app.category(),
        app.installed_size(),
        app.license_key(),
        app.download_token()};
    if (has_app(primary))
    {
        _apps.at(primary) = data;
    }
    else
    {
        _apps.emplace(primary, data);
    }
}

void app_db_t::delete_app(const apps_table_primary_t& primary)
{
    _apps.erase(primary);
}

bool app_db_t::has_app(const apps_table_primary_t& primary) const noexcept
{
    return _apps.find(primary) != _apps.cend();
}

std::vector<apps_table_entry_t> app_db_t::all_apps() const
{
    auto res = std::vector<apps_table_entry_t>{};
    for (decltype(auto) app : _apps)
    {
        res.emplace_back(apps_table_entry_t{app.first, app.second});
    }
    return res;
}

void app_db_t::insert_instance(const instances_table_entry_t& entry)
{
    decltype(auto) primary = static_cast<const instances_table_primary_t&>(entry);
    decltype(auto) data = static_cast<const instances_table_data_t&>(entry);
    if (has_instance(primary))
    {
        _instances.at(primary) = data;
    }
    else
    {
        _instances.emplace(primary, data);
    }
}

void app_db_t::delete_instance(const instances_table_primary_t& primary)
{
    _instances.erase(primary);
}

bool app_db_t::has_instance(const instances_table_primary_t& primary) const noexcept
{
    return _instances.find(primary) != _instances.cend();
}

std::vector<instances_table_entry_t> app_db_t::all_instances() const
{
    auto res = std::vector<instances_table_entry_t>{};
    for (decltype(auto) instance : _instances)
    {
        res.emplace_back(instances_table_entry_t{instance.first, instance.second});
    }
    return res;
}

std::vector<instances_table_entry_t> app_db_t::instances(const std::string& app) const
{
    auto res = std::vector<instances_table_entry_t>{};
    for (decltype(auto) instance : _instances)
    {
        if (instance.second.app == app)
        {
            res.emplace_back(instances_table_entry_t{instance.first, instance.second});
        }
    }
    return res;
}

std::vector<instances_table_entry_t> app_db_t::instances(const std::string& app, const std::string& version) const
{
    auto res = std::vector<instances_table_entry_t>{};
    for (decltype(auto) instance : _instances)
    {
        if (instance.second.app == app && instance.second.version == version)
        {
            res.emplace_back(instances_table_entry_t{instance.first, instance.second});
        }
    }
    return res;
}

std::optional<apps_table_entry_t> app_db_t::query_app(const apps_table_primary_t& primary) const noexcept
{
    if (has_app(primary))
    {
        decltype(auto) data = _apps.at(primary);
        return apps_table_entry_t{primary, data};
    }
    return std::nullopt;
}

std::optional<instances_table_entry_t> app_db_t::query_instance(const instances_table_primary_t& primary) const noexcept
{
    if (has_instance(primary))
    {
        decltype(auto) data = _instances.at(primary);
        return instances_table_entry_t{primary, data};
    }
    return std::nullopt;
}

void app_db_t::cache_db()
{
    auto apps = std::vector<apps_table_entry_t>{};
    select_all(apps_table_name, &select_apps_callback, &apps);
    for (decltype(auto) app : apps)
    {
        _apps.emplace(static_cast<apps_table_primary_t>(app), app);
    }

    auto instances = std::vector<instances_table_entry_t>{};
    select_all(instances_table_name, &select_instances_callback, &instances);
    for (decltype(auto) instance : instances)
    {
        _instances.emplace(static_cast<instances_table_primary_t>(instance), instance);
    }

    query_user_version();
}

void app_db_t::migrate_db()
{
    if (user_version() < CURRENT_USER_VERSION)
    {
        // There is currently no need to check 'from' and 'to' version
        for (decltype(auto) instance : _instances)
        {
            if (instance.second.networks.empty())
            {
                instance.second.networks.emplace_back("flecs");
            }
        }
        persist();
        _user_version = CURRENT_USER_VERSION;
    }
}

int app_db_t::persist()
{
    const auto path_old = std::filesystem::path{_path};
    const auto path_new = std::filesystem::path{std::string{_path} + ".sav"};

    close();

    auto ec = std::error_code{};
    std::filesystem::rename(path_old, path_new, ec);

    open(_path.c_str(), SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX, nullptr);
    create_app_table();
    create_instances_table();
    set_user_version();
    auto res = SQLITE_OK;

    for (decltype(auto) app : _apps)
    {
        res = insert(
            apps_table_name,
            app.first.app,
            app.first.version,
            app.second.status,
            app.second.desired,
            app.second.category,
            app.second.installed_size,
            app.second.license_key,
            app.second.download_token);
        if (res != SQLITE_OK)
        {
            return res;
        }
    }

    for (decltype(auto) instance : _instances)
    {
        res = insert(
            instances_table_name,
            instance.first.id,
            instance.second.app,
            instance.second.version,
            instance.second.status,
            instance.second.desired,
            instance.second.description,
            stringify_delim(',', instance.second.networks),
            stringify_delim(',', instance.second.ips),
            instance.second.flags);
        if (res != SQLITE_OK)
        {
            return res;
        }
    }
    return res;
}

} // namespace FLECS
