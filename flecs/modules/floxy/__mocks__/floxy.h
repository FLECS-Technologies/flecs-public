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

#include <gmock/gmock.h>

#include "flecs/modules/instances/types/instance.h"
#include "flecs/modules/instances/types/instance_id.h"
#include "flecs/modules/module_base/module.h"

namespace flecs {
namespace module {
namespace impl {
class floxy_t
{
public:
    ~floxy_t() = default;
};
} // namespace impl

class floxy_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~floxy_t() = default;

    MOCK_METHOD(
        (result_t),
        load_instance_reverse_proxy_config,
        (const std::string&, const std::string&, const instances::id_t&, std::vector<std::uint16_t>&),
        ());
    MOCK_METHOD((result_t), delete_reverse_proxy_configs, (std::shared_ptr<instances::instance_t>), ());
    MOCK_METHOD((result_t), delete_server_proxy_configs, (std::shared_ptr<instances::instance_t>), ());

protected:
    floxy_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    std::unique_ptr<impl::floxy_t> _impl;
};

} // namespace module
} // namespace flecs
