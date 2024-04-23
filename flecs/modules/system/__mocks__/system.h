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

#include <map>
#include <string>
#include <vector>

#include "flecs/modules/module_base/module.h"
#include "flecs-core-rs/src/lib.rs.h"

namespace flecs {

enum netif_type_t {
    UNKNOWN,
    WIRED,
    WIRELESS,
    LOCAL,
    BRIDGE,
    VIRTUAL,
};

struct ipaddr_t
{
    std::string addr;
    std::string subnet_mask;
};

struct netif_t
{
    std::string mac;
    netif_type_t type;
    std::vector<ipaddr_t> ipv4_addr;
    std::vector<ipaddr_t> ipv6_addr;
    std::string gateway;
};

namespace module {

class system_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    MOCK_METHOD((std::map<std::string, NetInfo>), get_network_adapters, (), (const));

protected:
    system_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    MOCK_METHOD((crow::response), ping, (), (const));
    MOCK_METHOD((crow::response), info, (), (const));
};

} // namespace module
} // namespace flecs
