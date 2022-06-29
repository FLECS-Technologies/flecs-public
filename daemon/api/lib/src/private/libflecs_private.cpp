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

#include "private/libflecs_private.h"

#include <cpr/cpr.h>

#include <filesystem>
#include <fstream>
#include <mutex>

#include "util/json/json.h"

namespace FLECS {
namespace Private {

template <typename Key, typename Value, typename... Args>
json_t build_json_impl(json_t& json, Key&& key, Value&& value, Args&&... args)
{
    json[key] = value;
    if constexpr (sizeof...(args))
    {
        return build_json_impl(json, std::forward<Args>(args)...);
    }
    return json;
}

template <typename... Args>
json_t build_json(Args&&... args)
{
    auto json = json_t{};
    return build_json_impl(json, std::forward<Args>(args)...);
}

libflecs_private_t::libflecs_private_t()
    : _base_url{}
    , _response{}
{}

libflecs_private_t::~libflecs_private_t()
{}

int libflecs_private_t::do_connect(std::string host, int port)
{
    do_disconnect();

    _base_url = cpr::Url{host + ":" + std::to_string(port)};

    return do_ping();
}

int libflecs_private_t::do_disconnect()
{
    _base_url = cpr::Url{""};

    return 0;
}

// app management
int libflecs_private_t::do_install_app(const std::string& app, const std::string& version, const std::string& license)
{
    auto body = build_json("app", app, "version", version, "licenseKey", license);
    return post("/app/install", body.dump().c_str());
}

int libflecs_private_t::do_uninstall_app(const std::string& app, const std::string& version)
{
    auto body = build_json("app", app, "version", version);
    return post("/app/uninstall", body.dump().c_str());
}

int libflecs_private_t::do_sideload_app_from_yaml(const std::string& yaml)
{
    auto body = build_json("appYaml", yaml);
    return put("/app/sideload", body.dump().c_str());
}

int libflecs_private_t::do_sideload_app_from_file(const std::filesystem::path& manifest_path)
{
    if (!std::filesystem::exists(manifest_path))
    {
        std::fprintf(stderr, "Specified manifest %s does not exist\n", manifest_path.c_str());
        return -1;
    }

    auto ec = std::error_code{};
    const auto file_size = std::filesystem::file_size(manifest_path, ec);
    if (ec)
    {
        std::fprintf(stderr, "Could not determine size of %s: %d\n", manifest_path.c_str(), ec.value());
        return -1;
    }

    auto data = std::make_unique<char[]>(file_size + 1);
    auto file = fopen(manifest_path.c_str(), "rb");
    if (!file)
    {
        std::fprintf(stderr, "Could not open %s for reading: %d\n", manifest_path.c_str(), errno);
        return -1;
    }
    const auto bytes_read = fread(data.get(), 1, file_size, file);
    fclose(file);
    if (bytes_read != file_size)
    {
        return -1;
    }

    auto body = build_json("appYaml", data.get());
    return put("/app/sideload", body.dump().c_str());
}

int libflecs_private_t::do_list_apps()
{
    return get("/app/list");
}

int libflecs_private_t::do_list_instances(const std::string& /*app*/, const std::string& /*version*/)
{
    return -1;
}

int libflecs_private_t::do_list_versions(const std::string& /*app*/)
{
    return -1;
}

// instance management
int libflecs_private_t::do_create_instance(
    const std::string& app,
    const std::string& version,
    const std::string& instanceName)
{
    auto body = build_json("app", app, "version", version, "instanceName", instanceName);
    return post("/instance/create", body.dump().c_str());
}

int libflecs_private_t::do_delete_instance(
    const std::string& instanceId,
    const std::string& app,
    const std::string& version)
{
    auto body = build_json("instanceId", instanceId, "app", app, "version", version);
    return post("/instance/delete", body.dump().c_str());
}

int libflecs_private_t::do_start_instance(
    const std::string& instanceId,
    const std::string& app,
    const std::string& version)
{
    auto body = build_json("instanceId", instanceId, "app", app, "version", version);
    return post("/instance/start", body.dump().c_str());
}

int libflecs_private_t::do_stop_instance(
    const std::string& instanceId,
    const std::string& app,
    const std::string& version)
{
    auto body = build_json("instanceId", instanceId, "app", app, "version", version);
    return post("/instance/stop", body.dump().c_str());
}

// system info
int libflecs_private_t::do_version()
{
    return get("/system/version");
}

int libflecs_private_t::do_ping()
{
    return get("/system/ping");
}

using command_callback_t = int (libflecs_private_t::*)(const std::vector<std::string>&);
struct command_t
{
    std::string command;
    command_callback_t cbk;
    std::vector<command_t> subcommands;
};

// string-based interface
int libflecs_private_t::do_run_command(const std::string& command, const std::vector<std::string>& args)
{
    const auto known_commands = std::vector<command_t>{
        {"app-manager",
         nullptr,
         {command_t{"list-apps", &libflecs_private_t::dispatch_list_apps, {}},
          command_t{"install", &libflecs_private_t::dispatch_install_app, {}},
          command_t{"uninstall", &libflecs_private_t::dispatch_uninstall_app, {}},
          command_t{"sideload", &libflecs_private_t::dispatch_sideload_app, {}},
          command_t{"create-instance", &libflecs_private_t::dispatch_create_instance, {}},
          command_t{"delete-instance", &libflecs_private_t::dispatch_delete_instance, {}},
          command_t{"start-instance", &libflecs_private_t::dispatch_start_instance, {}},
          command_t{"stop-instance", &libflecs_private_t::dispatch_stop_instance, {}}}},
        {"system", nullptr, {command_t{"ping", &libflecs_private_t::dispatch_ping, {}}}},
        {"version", &libflecs_private_t::dispatch_version, {}}};

    // search top-level command
    auto i = known_commands.cbegin();
    for (; i != known_commands.cend(); ++i)
    {
        if (i->command == command)
        {
            // command found and has callback -> done
            if (i->cbk)
            {
                return std::invoke(i->cbk, this, args);
            }
            // command found and has no callback -> continue searching subcommands
            else
            {
                break;
            }
        }
    }
    // command not found or no subcommand specified -> return
    if (i == known_commands.cend() || args.empty())
    {
        return -1;
    }

    // search in subcommands
    for (auto j = i->subcommands.cbegin(); j != i->subcommands.cend(); ++j)
    {
        if (j->command == args[0] && j->cbk)
        {
            auto cmd_args = std::vector<std::string>{args.begin() + 1, args.end()};
            return std::invoke(j->cbk, this, cmd_args);
        }
    }

    return -1;
}

#define ARG_OR_EMPTY(num) args.size() > num ? args[num] : ""

int libflecs_private_t::dispatch_install_app(const std::vector<std::string>& args)
{
    return do_install_app(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1), ARG_OR_EMPTY(2));
}

int libflecs_private_t::dispatch_uninstall_app(const std::vector<std::string>& args)
{
    return do_uninstall_app(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1));
}

int libflecs_private_t::dispatch_sideload_app(const std::vector<std::string>& args)
{
    return do_sideload_app_from_file(ARG_OR_EMPTY(0));
}

int libflecs_private_t::dispatch_list_apps(const std::vector<std::string>& /*args*/)
{
    return do_list_apps();
}

int libflecs_private_t::dispatch_list_instances(const std::vector<std::string>& args)
{
    return do_list_instances(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1));
}

int libflecs_private_t::dispatch_list_versions(const std::vector<std::string>& args)
{
    return do_list_versions(ARG_OR_EMPTY(0));
}

int libflecs_private_t::dispatch_create_instance(const std::vector<std::string>& args)
{
    return do_create_instance(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1), ARG_OR_EMPTY(2));
}

int libflecs_private_t::dispatch_delete_instance(const std::vector<std::string>& args)
{
    return do_delete_instance(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1), ARG_OR_EMPTY(2));
}

int libflecs_private_t::dispatch_start_instance(const std::vector<std::string>& args)
{
    return do_start_instance(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1), ARG_OR_EMPTY(2));
}

int libflecs_private_t::dispatch_stop_instance(const std::vector<std::string>& args)
{
    return do_stop_instance(ARG_OR_EMPTY(0), ARG_OR_EMPTY(1), ARG_OR_EMPTY(2));
}

int libflecs_private_t::dispatch_version(const std::vector<std::string>& /*args*/)
{
    return do_version();
}

int libflecs_private_t::dispatch_ping(const std::vector<std::string>& /*args*/)
{
    return do_ping();
}

// retrieve actual HTTP status code
int libflecs_private_t::do_response_code() const noexcept
{
    return static_cast<int>(_response.status_code);
}

// retrieve response as formatted JSON string
std::string libflecs_private_t::do_json_response() const noexcept
{
    return _response.text;
}

cpr::Url libflecs_private_t::build_url(const std::string& endpoint)
{
    return cpr::Url{_base_url.str() + endpoint};
}

int libflecs_private_t::get(const std::string& endpoint)
{
    if (_base_url.str().empty())
    {
        return -1;
    }

    _response = cpr::Get(build_url(endpoint));

    return !_response.error ? 0 : -1;
}

int libflecs_private_t::post(const std::string& endpoint, const char* data)
{
    if (_base_url.str().empty())
    {
        return -1;
    }

    _response = cpr::Post(build_url(endpoint), cpr::Header{{"Content-type", "application/json"}}, cpr::Body{data});

    return !_response.error ? 0 : -1;
}

int libflecs_private_t::put(const std::string& endpoint, const char* data)
{
    if (_base_url.str().empty())
    {
        return -1;
    }

    _response = cpr::Put(build_url(endpoint), cpr::Header{{"Content-type", "application/x-yaml"}}, cpr::Body{data});

    return !_response.error ? 0 : -1;
}

} // namespace Private
} // namespace FLECS
