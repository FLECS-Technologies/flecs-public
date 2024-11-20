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

#include <netinet/in.h>
#include <sys/socket.h>

#include <fstream>
#include <string>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/modules/apps/types/app.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/modules/instances/instances.h"
#include "flecs/modules/instances/types/instance.h"
#include "flecs/util/process/process.h"

namespace flecs {
namespace module {
namespace impl {

namespace {
auto get_random_free_port() //
    -> std::optional<std::uint16_t>
{
    auto serv_addr = sockaddr_in{};
    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        return {};
    }
    bzero((char*)&serv_addr, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    serv_addr.sin_addr.s_addr = INADDR_ANY;
    serv_addr.sin_port = htons(0);
    if (bind(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
        return {};
    }
    socklen_t len = sizeof(serv_addr);
    if (getsockname(sockfd, (struct sockaddr*)&serv_addr, &len) < 0) {
        close(sockfd);
        return {};
    }
    close(sockfd);
    return ntohs(serv_addr.sin_port);
}
} // namespace

floxy_t::floxy_t(flecs::module::floxy_t* parent)
    : _parent{parent}
{}

auto floxy_t::do_init() //
    -> void
{
    clear_server_configs();
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

auto floxy_t::clear_server_configs(const fs::path base_path) //
    -> void
{
    const auto dir = base_path / "floxy" / "servers";
    if (fs::exists(dir) && fs::is_directory(dir)) {
        for (const auto& entry : fs::directory_iterator(dir)) {
            try {
                if ((entry.is_regular_file() || entry.is_symlink()) && entry.path().extension() == ".conf") {
                    fs::remove(entry.path());
                }
            } catch (const fs::filesystem_error&) {
            }
        }
    }
}

auto floxy_t::build_instance_config_path(
    const std::string& app_name, const instances::id_t& instance_id, const fs::path base_path) //
    -> fs::path
{
    const auto dir = base_path / "floxy" / "instances";
    auto file_name = app_name + "-" + instance_id.hex() + ".conf";
    return dir / file_name;
}

auto floxy_t::build_server_config_path(
    const std::string& app_name,
    const instances::id_t& instance_id,
    std::uint16_t host_port,
    const fs::path base_path) //
    -> fs::path
{
    const auto dir = base_path / "floxy" / "servers";
    auto file_name = app_name + "-" + instance_id.hex() + "_" + std::to_string(host_port) + ".conf";
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
    const instances::id_t& instance_id, const std::string& instance_address, std::uint16_t dest_port) //
    -> std::string
{
    auto location = "/v2/instances/" + instance_id.hex() + "/editor/" + std::to_string(dest_port);
    auto upstream = instance_address + ":" + std::to_string(dest_port);
    auto first = R"(
location )";
    auto second = R"( {
   server_name_in_redirect on;
   return 301 $request_uri/;

   location ~ ^)";
    auto third = R"(/(.*) {
      set $upstream http://)";
    auto fourth = R"(/$1;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
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

auto floxy_t::create_server_config(
    const std::string& instance_address, std::uint16_t host_port, std::uint16_t dest_port) //
    -> std::string
{
    auto upstream = instance_address + ":" + std::to_string(dest_port);
    auto first = R"(
server {
   listen )";
    auto second = R"(;
   location / {
      set $upstream http://)";
    auto third = R"(;
      proxy_pass $upstream;

      proxy_http_version 1.1;

      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection $connection_upgrade;
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_set_header X-Real-IP $remote_addr;
      proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
      proxy_set_header X-Forwarded-Host $host;
      proxy_set_header X-Forwarded-Port $server_port;

      client_max_body_size 0;
      client_body_timeout 30m;
   }
})";
    return first + std::to_string(host_port) + second + upstream + third;
}

auto floxy_t::load_reverse_proxy_config(const std::string& content, const fs::path& file_path) //
    -> result_t
{
    auto ec = std::error_code{};
    fs::create_directories(file_path.parent_path(), ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }

    // Reload only if config does not exist or is different
    auto reload_necessary = true;
    auto existing_config = std::ifstream{file_path, std::ios::in | std::ios::ate};
    if (existing_config.is_open()) {
        std::streamsize size = existing_config.tellg();
        existing_config.seekg(0, std::ios::beg);
        std::string existing_content(size, '\0');
        existing_config.read(&existing_content[0], size);
        existing_config.close();
        reload_necessary = content != existing_content;
    }

    if (reload_necessary) {
        auto config_file = std::ofstream{file_path, std::ios::out | std::ios::trunc};
        if (!config_file.good()) {
            auto message = std::string("Could not open ") + file_path.c_str() + " for writing";
            return {-1, message};
        }
        config_file << content;
        config_file.flush();
        config_file.close();
        return reload_floxy_config();
    }
    return {0, {}};
}

auto floxy_t::do_load_instance_reverse_proxy_config(
    const std::string& ip_address,
    const std::string& app_name,
    const instances::id_t& instance_id,
    std::vector<std::uint16_t>& dest_ports) //
    -> result_t
{
    std::sort(dest_ports.begin(), dest_ports.end());
    const auto config_path = build_instance_config_path(app_name, instance_id);
    auto config = std::string{};
    for (auto dest_port : dest_ports) {
        config += create_instance_config(instance_id, ip_address, dest_port);
    }

    return load_reverse_proxy_config(config, config_path);
}

auto floxy_t::delete_reverse_proxy_config(const fs::path& file_path, bool reload) //
    -> result_t
{
    if (!fs::remove(file_path)) {
        auto message = std::string("Could not delete ") + file_path.c_str();
        return {-1, message};
    }
    if (reload) {
        return reload_floxy_config();
    }
    return {0, {}};
}

auto floxy_t::delete_server_config(
    const std::string& app_name, const instances::id_t& instance_id, std::uint16_t host_port, bool reload) //
    -> result_t
{
    return delete_reverse_proxy_config(build_server_config_path(app_name, instance_id, host_port), reload);
}

auto floxy_t::do_redirect_editor_request(instances::id_t instance_id, std::uint16_t port) //
    -> crow::response
{
    const auto instances_api = dynamic_cast<module::instances_t*>(api::query_module("instances").get());
    if (!instances_api) {
        return {crow::status::INTERNAL_SERVER_ERROR, {}};
    }
    auto instance = instances_api->query(instance_id);
    if (!instance) {
        auto json = json_t{{"additionalInfo", "Instance not found"}};
        return {crow::status::NOT_FOUND, json.dump()};
    }
    const auto& editors = instance->app()->manifest()->editors();
    auto editor = editors.find(port);
    if (editor == editors.end()) {
        auto json = json_t{{"additionalInfo", "Unknown port"}};
        return {crow::status::NOT_FOUND, json.dump()};
    }
    if (editor->second.supports_reverse_proxy()) {
        auto response = json_t{{"additionalInfo", "Editor supports reverse proxy -> use floxy"}};
        return crow::response{crow::status::BAD_REQUEST, response.dump()};
    }
    if (!instances_api->is_running(instance)) {
        auto json = json_t{{"additionalInfo", "Instance is not running"}};
        return crow::response{crow::status::BAD_REQUEST, json.dump()};
    }
    auto response = crow::response{};
    auto& mapping = instance->editor_port_mapping();
    auto result = mapping.find(port);
    if (result != mapping.end()) {
        response.moved(":" + to_string(result->second));
    } else {
        response = redirect_editor_request_to_free_port(instance, port);
    }
    return response;
}

auto floxy_t::do_delete_reverse_proxy_configs(std::shared_ptr<instances::instance_t> instance) //
    -> result_t
{
    auto app_name = instance->app()->key().name();
    do_delete_server_proxy_configs(instance, false);
    return delete_reverse_proxy_config(build_instance_config_path(app_name, instance->id()));
}

auto floxy_t::do_delete_server_proxy_configs(std::shared_ptr<instances::instance_t> instance, bool reload) //
    -> result_t
{
    auto app_name = instance->app()->key().name();
    for (auto [_, host_port] : instance->editor_port_mapping()) {
        delete_server_config(app_name, instance->id(), host_port, false);
    }
    if (reload) {
        return reload_floxy_config();
    }
    return {0, {}};
}

auto floxy_t::redirect_editor_request_to_free_port(
    std::shared_ptr<instances::instance_t> instance, std::uint16_t dest_port) //
    -> crow::response
{
    auto host_port = get_random_free_port();
    if (!host_port.has_value()) {
        auto json = json_t{{"additionalInfo", "No free port available"}};
        return crow::response{crow::status::INTERNAL_SERVER_ERROR, json.dump()};
    }
    auto config_path =
        build_server_config_path(instance->app()->key().name(), instance->id(), host_port.value());
    std::optional<std::string> instance_ip;
    for (const auto& network : instance->networks()) {
        if (network.network_name == "flecs") {
            instance_ip = network.ip_address;
            break;
        }
    }
    if (!instance_ip.has_value()) {
        auto json = json_t{{"additionalInfo", "Instance not connected to network"}};
        return crow::response{crow::status::INTERNAL_SERVER_ERROR, json.dump()};
    }
    auto config_content = create_server_config(instance_ip.value(), host_port.value(), dest_port);
    auto [res, message] = load_reverse_proxy_config(config_content, config_path);
    if (res != 0) {
        auto json = json_t{{"additionalInfo", "Could not load reverse proxy config: " + message}};
        return crow::response{crow::status::INTERNAL_SERVER_ERROR, json.dump()};
    }
    instance->set_editor_port_mapping(host_port.value(), dest_port);
    auto response = crow::response{};
    response.moved(":" + to_string(host_port.value()));
    return response;
}

} // namespace impl
} // namespace module
} // namespace flecs
