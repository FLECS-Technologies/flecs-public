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
    auto docker_process = process_t{};

    docker_process.arg("run");
    docker_process.arg("--rm");
    docker_process.arg("-d");
    docker_process.arg("-p");
    docker_process.arg("50005:80");
    docker_process.arg("--network");
    docker_process.arg("flecs");
    docker_process.arg("--name");
    docker_process.arg("floxy");
    docker_process.arg("-v");
    docker_process.arg("/var/lib/flecs/floxy:/floxy");
    docker_process.arg("floxy");

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        std::cerr << "Failed to start floxy" << std::endl;
    }
}

auto floxy_t::do_deinit() //
    -> void
{
    auto docker_process = process_t{};

    docker_process.arg("stop");
    docker_process.arg("floxy");

    docker_process.spawnp("docker");
    docker_process.wait(false, true);
    if (docker_process.exit_code() != 0) {
        std::cerr << "Failed to stop floxy" << std::endl;
    }
}

auto floxy_t::build_config_path(const std::string& app_name, const instances::id_t& instance_id, const fs::path base_path) //
    -> fs::path
{
    const auto dir = base_path / "floxy";
    auto file_name = app_name + "-" + instance_id.hex() + ".conf";
    return dir / file_name;
}

auto floxy_t::reload_floxy_config() //
    -> result_t
{
    auto docker_process = process_t{};

    docker_process.arg("container");
    docker_process.arg("exec");
    docker_process.arg("floxy");
    docker_process.arg("nginx");
    docker_process.arg("-s");
    docker_process.arg("reload");

    nginx_process.spawnp("nginx");
    nginx_process.wait(true, true);
    if (nginx_process.exit_code() != 0) {
        return {-1, "Failed to reload floxy config"};
    }
    return {0, {}};
}

auto floxy_t::create_instance_config(
    const std::string& app_name, const std::string& instance_name, uint16_t dest_port) //
    -> std::string
{
    auto location = "/editor/" + app_name + "/" + instance_name + "/" + std::to_string(dest_port);
    auto upstream = "flecs-" + app_name + ":" + std::to_string(dest_port);
    auto first = R"(
location )";
    auto second = R"( {
   return 301 $scheme://$host/editor/$request_uri;

   location )";
    auto third = R"(/ {
      set $upstream http://)";
    auto fourth = R"(/;
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

auto floxy_t::do_load_instance_reverse_proxy_config(const std::string& app_name, const instances::id_t& instance_id, uint16_t dest_port) //
    -> result_t
{
    const auto config_path = build_config_path(app_name, instance_id);
    auto ec = std::error_code{};
    fs::create_directories(config_path.parent_path(), ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }
    auto config_file = std::ofstream{config_path, std::ios::out | std::ios::trunc};
    if (!config_file.good()) {
        auto message = std::string("Could not open ") + config_path.c_str() + " for writing";
        return {-1, message};
    }
    auto config = create_instance_config(
        app_name,
        instance_id.hex(),
        dest_port);
    config_file << config << std::endl;
    reload_floxy_config();
    return {0, {}};
}

auto floxy_t::do_delete_instance_reverse_proxy_config(const std::string& app_name, const instances::id_t& instance_id) //
    -> result_t
{
    auto path = build_config_path(app_name, instance_id);
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
