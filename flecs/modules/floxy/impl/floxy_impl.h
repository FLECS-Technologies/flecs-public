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

#pragma once

#include <string>

#include "flecs/modules/floxy/floxy.h"
#include "flecs/modules/instances/types/instance_id.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace module {
namespace impl {

class floxy_t
{
    friend class flecs::module::floxy_t;

private:
    floxy_t() = default;

    auto do_load_instance_reverse_proxy_config(
        const std::string& ip_address,
        const std::string& app_name,
        const instances::id_t& instance_id,
        std::vector<std::uint16_t>& dest_ports) //
        -> result_t;

    auto do_delete_reverse_proxy_configs(std::shared_ptr<instances::instance_t> instance) //
        -> result_t;

    auto do_delete_server_proxy_configs(
        std::shared_ptr<instances::instance_t> instance) //
        -> result_t;

    auto do_redirect_editor_request(instances::id_t instance_id, std::uint16_t port) //
        -> crow::response;
};

} // namespace impl
} // namespace module
} // namespace flecs
