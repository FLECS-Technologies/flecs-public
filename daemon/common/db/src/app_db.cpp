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

app_db_t::app_db_t()
    : app_db_t{app_db_path}
{}

app_db_t::app_db_t(std::string path)
    : sqlite3_db_t{path.c_str(), SQLITE_OPEN_READONLY | SQLITE_OPEN_NOMUTEX, nullptr}
    , _path{path}
{}

std::vector<apps_table_entry_t> app_db_t::all_apps()
{
    auto apps = std::vector<apps_table_entry_t>{};
    select_all(apps_table_name, &select_apps_callback, &apps);
    return apps;
}

std::vector<instances_table_entry_t> app_db_t::all_instances()
{
    auto instances = std::vector<instances_table_entry_t>{};
    select_all(instances_table_name, &select_instances_callback, &instances);
    return instances;
}

} // namespace FLECS
