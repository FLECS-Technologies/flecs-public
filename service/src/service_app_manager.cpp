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

#include "service/service_app_manager.h"

#include "service/app.h"
#include "service/app_db.h"
#include "service/app_status.h"
#include "util/container/map_constexpr.h"
#include "util/curl_easy_ext/curl_easy_ext.h"
#include "util/process/process.h"

#include "external/yaml-cpp-0.7.0/include/yaml-cpp/yaml.h"

#include <iomanip>
#include <iostream>
#include <random>
#include <sstream>
#include <string>
#include <vector>

#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>

namespace FLECS {

service_app_manager::service_app_manager()
{
    curl_global_init(CURL_GLOBAL_DEFAULT);
}

service_app_manager::~service_app_manager()
{
    curl_global_cleanup();
}

int service_app_manager::do_process(int argc, char** argv)
{
    if (argc < 1)
    {
        return -1;
    }
    const auto action = argv[0];

    using action_callback_t = int (service_app_manager::*)(int, char**);
    using action_callback_table_t = FLECS::map_c<const char*, action_callback_t, 8, string_comparator>;
    constexpr action_callback_table_t action_callbacks = {{
        std::make_pair("install",   &service_app_manager::install),
        std::make_pair("uninstall", &service_app_manager::uninstall),
        std::make_pair("create-instance", &service_app_manager::create_instance),
        std::make_pair("delete-instance", &service_app_manager::delete_instance),
        std::make_pair("start-instance", &service_app_manager::start_instance),
        std::make_pair("stop-instance", &service_app_manager::stop_instance),
        std::make_pair("list-apps", &service_app_manager::list_apps),
        std::make_pair("list-instances", &service_app_manager::list_instances),
    }};

    const auto it = action_callbacks.find(action);
    if (it != action_callbacks.end())
    {
        return std::invoke(it->second, this, argc, argv);
    }

    return -1;
}

int service_app_manager::install(int argc, char** argv)
{
    if (argc < 3)
    {
        return -1;
    }
    const auto app_name = argv[1];
    const auto version = argv[2];

    return do_install(app_name, version);
}

int service_app_manager::uninstall(int argc, char** argv)
{
    if (argc < 3)
    {
        return -1;
    }
    const auto app_name = argv[1];
    const auto version = argv[2];

    return do_uninstall(app_name, version);
}

int service_app_manager::create_instance(int argc, char** argv)
{
    if (argc < 3)
    {
        return -1;
    }
    const auto app_name = argv[1];
    const auto version = argv[2];
    const auto description = argc > 3 ? argv[3] : "";

    return do_create_instance(app_name, version, description);
}

int service_app_manager::delete_instance(int argc, char** argv)
{
    if (argc < 2)
    {
        return -1;
    }
    const auto id = argv[1];
    return do_delete_instance(id);
}

int service_app_manager::start_instance(int argc, char** argv)
{
    if (argc < 2)
    {
        return -1;
    }
    const auto id = argv[1];

    return do_start_instance(id);
}

int service_app_manager::stop_instance(int argc, char** argv)
{
    if (argc < 2)
    {
        return -1;
    }
    const auto id = argv[1];

    return do_stop_instance(id);
}

int service_app_manager::list_apps(int /*argc*/, char** /*argv*/)
{
    return 0;
}

int service_app_manager::list_instances(int /*argc*/, char** /*argv*/)
{
    return 0;
}

int service_app_manager::do_install(const std::string& app_name, const std::string& version)
{
    auto status = NOT_INSTALLED;
    const auto desired = INSTALLED;

    const auto path = build_manifest_path(app_name, version);
    const auto manifest = fopen(path.c_str(), "w");
    if (manifest == nullptr)
    {
        std::cerr << "Could not open " << path << " for writing" << std::endl;
        return -1;
    }

    const auto fd = fileno(manifest);
    if (fd < 0)
    {
        std::cerr << "Could not get fd for " << path << std::endl;
        return -1;
    }

    const auto url = build_manifest_url(app_name, version);
    curl_easy_ext curl { url.c_str(), fd };
    if (!curl)
    {
        std::cerr << "Could not initialize curl_easy_ext" << std::endl;
        return -1;
    }

    const auto curl_res = curl.perform();
    fclose(manifest);
    if (curl_res != CURLE_OK)
    {
        std::cerr << "Could not download app manifest: " << curl_res << std::endl;
        return -1;
    }

    status = MANIFEST_DOWNLOADED;

    app_t app { path };
    if (!app.yaml_loaded())
    {
        return -1;
    }

    auto docker_process = process_t {};
    docker_process.spawnp("docker", "pull", app.image_with_tag());
    docker_process.wait(true, true);
    if (docker_process.exit_code() != 0)
    {
        return -1;
    }

    status = IMAGE_DOWNLOADED;

    auto app_db = app_db_t {};

    status = INSTALLED;

    const auto sqlite_res = app_db.insert_app(apps_table_entry_t { app_name, version, status, desired, "", 0 });
    if (sqlite_res != SQLITE_OK)
    {
        return -1;
    }

    return 0;
}

int service_app_manager::do_uninstall(const std::string& app_name, const std::string& version)
{
    auto app_db = app_db_t {};

    auto app_entry = app_db.query_app( { app_name, version} );
    if ((app_entry.app != app_name) || (app_entry.version != version) || (app_entry.status != 'i'))
    {
        std::cerr << "Could not uninstall " << app_name << " (" << version << "): not installed" << std::endl;
        return -1;
    }

    const auto path = build_manifest_path(app_name, version);

    auto app = app_t { path };
    if (!app.yaml_loaded())
    {
        return -1;
    }

    const auto instances = app_db.query_instances( { app_name, version } );
    for (auto& instance : instances)
    {
        const auto res = do_stop_instance(instance.id);
        if (res < 0)
        {
            return -1;
        }
        app_db.delete_instance( { app_name });
    }

    const auto image = app.image_with_tag();
    auto docker_process = process_t {};
    docker_process.spawnp("docker", "rmi", "-f", image);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return -1;
    }

    const auto res = unlink(path.c_str());
    if (res < 0)
    {
        std::cerr << "Could not delete manifest: " << errno << std::endl;
        return -1;
    }

    app_db.delete_app( { app_name, version} );

    return 0;
}

int service_app_manager::do_create_instance(
    const std::string& app_name,
    const std::string& version,
    const std::string& description)
{
    auto app_db = app_db_t {};

    const auto app_entry = app_db.query_app({ app_name, version});
    if ((app_entry.app != app_name) || (app_entry.version != version) || (app_entry.status != 'i'))
    {
        std::cerr << "Could not create instance: " << app_name << " (" << version << ") not installed" << std::endl;
        return -1;
    }

    auto seed = std::random_device {};
    auto generator = std::mt19937 { seed() };
    auto distribution = std::uniform_int_distribution {
        std::numeric_limits<std::uint32_t>::min(),
        std::numeric_limits<std::uint32_t>::max()
    };

    const auto id = distribution(generator);
    auto ss = std::stringstream {};
    ss << std::hex << std::setw(8) << std::setfill('0') << id;

    const auto path = build_manifest_path(app_name, version);
    app_t app { path };
    if (!app.yaml_loaded())
    {
        return -1;
    }

    for (const auto& volume : app.volumes())
    {
        auto docker_process = process_t {};
        const auto name = std::string { "flecs-" } + ss.str() + "-" + volume.first;
        docker_process.spawnp("docker", "volume", "create", name);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            std::cerr << "Could not create docker volume " << name << std::endl;
            return -1;
        }
    }

    for (const auto& network : app.networks())
    {
        auto docker_process = process_t {};
        docker_process.spawnp("docker", "network", "create", network);
        docker_process.wait(false, true);
        if (docker_process.exit_code() != 0)
        {
            std::cerr << "Could not create docker network " << network << std::endl;
            return -1;
        }
    }

    const auto sqlite_res = app_db.insert_instance({ ss.str().c_str(), app_name, version, description, 0 });
    if (sqlite_res != SQLITE_OK)
    {
        return -1;
    }
    return 0;
}

int service_app_manager::do_delete_instance(const std::string& /*id*/)
{
    return 0;
}

int service_app_manager::do_start_instance(const std::string& id)
{
    auto app_db = app_db_t {};

    const auto instance = app_db.query_instance( { id } );

    return 0;
}

int service_app_manager::do_stop_instance(const std::string& id)
{
    auto docker_process = process_t {};
    const auto name = std::string { "flecs" } + "-" + id;
    docker_process.spawnp("docker", "stop", name);
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0)
    {
        return -1;
    }
    return 0;
}

int service_app_manager::do_list_apps()
{
    return 0;
}

int service_app_manager::do_list_instances()
{
    return 0;
}

std::string service_app_manager::build_manifest_url(
    const std::string& app_name,
    const std::string& version) const
{
    auto url = std::string { "https://3ef7dc4.online-server.cloud/manifests/apps/" };

    url.append(app_name);
    url.append("/");
    url.append(version);
    url.append("/");
    url.append("manifest.yml");

    return url;
}

std::string service_app_manager::build_manifest_path(
    const std::string& app_name,
    const std::string& version) const
{
    auto path = std::string { "/var/lib/flecs/apps" };

    mkdir(path.c_str(), 0755);
    path.append("/");
    path.append(app_name);
    mkdir(path.c_str(), 0755);
    path.append("/");
    path.append(version);
    mkdir(path.c_str(), 0755);
    path.append("/");
    path.append("manifest.yml");

    return path;
}

} // namespace FLECS
