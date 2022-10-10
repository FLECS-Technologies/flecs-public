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

#ifndef E72B4073_69B1_4AE8_BCEC_210EBF10489E
#define E72B4073_69B1_4AE8_BCEC_210EBF10489E

#include <map>
#include <string>
#include <vector>

#include "module_base/module.h"

namespace FLECS {

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

class module_system_t : public module_t
{
public:
    auto get_network_adapters() const -> std::map<std::string, netif_t>;

protected:
    friend class module_factory_t;

    module_system_t();

    auto ping(json_t& response) const //
        -> crow::response;

private:
    auto do_init() //
        -> void override;
};

} // namespace FLECS

#endif // E72B4073_69B1_4AE8_BCEC_210EBF10489E
