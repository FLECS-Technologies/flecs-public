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

#pragma once

#include "flecs/modules/console/types/session_id.h"
#include "flecs/modules/instances/types/instance_id.h"
#include "flecs/modules/module_base/module.h"

namespace flecs {
namespace module {
namespace impl {
class floxy_t;
} // namespace impl

class floxy_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~floxy_t();

    auto load_instance_reverse_proxy_config(const std::string& ip_address, const std::string& app_name, const instances::id_t& instance_id, std::vector<uint16_t>& dest_ports) //
        -> result_t;

    auto delete_instance_reverse_proxy_config(const std::string& app_name, const instances::id_t& instance_id) //
        -> result_t;

protected:
    floxy_t();

    auto do_init() //
        -> void override;

    auto do_deinit() //
        -> void override;

    auto redirect_editor_request(instances::id_t instance_id, uint16_t port) //
        -> crow::response;

    std::unique_ptr<impl::floxy_t> _impl;
};

} // namespace module
} // namespace flecs
