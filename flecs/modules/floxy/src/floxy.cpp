// Copyright 2021-2024 FLECS Technologies GmbH
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

#include "flecs/modules/floxy/floxy.h"

#include "flecs/modules/floxy/impl/floxy_impl.h"
#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {


floxy_t::floxy_t()
    : _impl{new impl::floxy_t{this}}
{}

floxy_t::~floxy_t() = default;

auto floxy_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/instances/<string>/editor/<uint>").methods("GET"_method)([this](const std::string& instance_id, uint64_t port) {
        if (port > std::numeric_limits<uint16_t>::max()) {
            auto response = json_t{{"additionalInfo", "Port out of limits (max = " + std::to_string(std::numeric_limits<uint16_t>::max()) + ")"}};
            return crow::response{crow::status::BAD_REQUEST, response.dump()};
        }
        return redirect_editor_request(instances::id_t(instance_id), port);
    });

    _impl->do_init();
}

auto floxy_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto floxy_t::load_instance_reverse_proxy_config(const std::string& ip_address, const std::string& app_name, const instances::id_t& instance_id, std::vector<uint16_t>& dest_ports) //
    -> result_t
{
    return _impl->do_load_instance_reverse_proxy_config(ip_address, app_name, instance_id, dest_ports);
}

auto floxy_t::delete_instance_reverse_proxy_config(const std::string& app_name, const instances::id_t& instance_id) //
    -> result_t
{
    return _impl->do_delete_instance_reverse_proxy_config(app_name, instance_id);
}

auto floxy_t::redirect_editor_request(instances::id_t instance_id, uint16_t port) -> crow::response
{
    return _impl->do_redirect_editor_request(instance_id, port);
}

} // namespace module
} // namespace flecs
