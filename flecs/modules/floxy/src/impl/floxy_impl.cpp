// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "flecs/modules/floxy/impl/floxy_impl.h"

#include <fstream>
#include <string>

#include "flecs/modules/factory/factory.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace module {
namespace impl {

floxy_t::floxy_t(flecs::module::floxy_t* parent)
    : _parent{parent}
{}


auto floxy_t::do_init() //
    -> void
{
    auto nginx_process = process_t{};

    nginx_process.arg("-c");
    nginx_process.arg(get_main_config_path());
    nginx_process.spawnp("nginx");
    nginx_process.wait(true, true);
    if (nginx_process.exit_code() != 0) {
        std::cerr << "Failed to start floxy" << std::endl;
    }
}

auto floxy_t::do_deinit() //
    -> void
{
    auto nginx_process = process_t{};

    nginx_process.arg("-c");
    nginx_process.arg(get_main_config_path());
    nginx_process.arg("-s");
    nginx_process.arg("quit");

    nginx_process.spawnp("nginx");
    nginx_process.wait(true, true);
    if (nginx_process.exit_code() != 0) {
        std::cerr << "Failed to stop floxy" << std::endl;
    }
}

auto floxy_t::build_instance_config_path(const std::string& app_name, const instances::id_t& instance_id, const fs::path base_path) //
    -> fs::path
{
    const auto dir = base_path / "floxy" / "instances";
    auto file_name = app_name + "-" + instance_id.hex() + ".conf";
    return dir / file_name;
}

auto floxy_t::get_main_config_path() //
    -> fs::path
{
    return "/etc/nginx/floxy.conf";
}

auto floxy_t::reload_floxy_config() //
    -> result_t
{
    auto nginx_process = process_t{};

    nginx_process.arg("-c");
    nginx_process.arg(get_main_config_path());
    nginx_process.arg("-s");
    nginx_process.arg("reload");

    nginx_process.spawnp("nginx");
    nginx_process.wait(true, true);
    if (nginx_process.exit_code() != 0) {
        return {-1, "Failed to reload floxy config"};
    }
    return {0, {}};
}

auto floxy_t::create_instance_config(
    const instances::id_t& instance_id, const std::string& instance_address, uint16_t dest_port) //
    -> std::string
{
    auto location = "/v2/instances/" + instance_id.hex() + "/editor";
    auto upstream = instance_address + ":" + std::to_string(dest_port);
    auto first = R"(
location )";
    auto second = R"( {
   return 301 $scheme://$host/editor/$request_uri;

   location ~ ^)";
    auto third = R"(/(.*) {
      set $upstream http://)";
    auto fourth = R"(/$1;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      #proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
}
)";
    return first + location + second + location + third + upstream + fourth;
}

auto floxy_t::do_load_instance_reverse_proxy_config(const std::string& ip_address, const std::string& app_name, const instances::id_t& instance_id, uint16_t dest_port) //
    -> result_t
{
    const auto config_path = build_instance_config_path(app_name, instance_id);
    auto ec = std::error_code{};
    fs::create_directories(config_path.parent_path(), ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }
    auto config = create_instance_config(
        instance_id,
        ip_address,
        dest_port);

    // Reload only if config does not exist or is different
    auto reload_necessary = true;
    auto existing_config = std::ifstream{config_path, std::ios::in | std::ios::ate};
    if (existing_config.is_open()) {
        std::streamsize size = existing_config.tellg();
        existing_config.seekg(0, std::ios::beg);
        std::string content(size, '\0');
        existing_config.read(&content[0], size);
        existing_config.close();
        reload_necessary = config != content;
    }

    if (reload_necessary) {
        auto config_file = std::ofstream{config_path, std::ios::out | std::ios::trunc};
        if (!config_file.good()) {
            auto message = std::string("Could not open ") + config_path.c_str() + " for writing";
            return {-1, message};
        }
        config_file << config;
        config_file.flush();
        config_file.close();
        return reload_floxy_config();
    }
    return {0, {}};
}

auto floxy_t::do_delete_instance_reverse_proxy_config(const std::string& app_name, const instances::id_t& instance_id) //
    -> result_t
{
    auto path = build_instance_config_path(app_name, instance_id);
    if (std::filesystem::remove(path)) {
        return {0, {}};
    } else {
        auto message = std::string("Could not delete ") + path.c_str();
        return {-1, message};
    }
}

} // namespace impl
} // namespace module
} // namespace flecs
