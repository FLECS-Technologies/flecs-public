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

#ifndef FF5A0392_A69F_47AE_9AD0_EE3218DC149C
#define FF5A0392_A69F_47AE_9AD0_EE3218DC149C

#include <map>
#include <string>
#include <string_view>

namespace FLECS {

enum class network_type_t {
    NONE,
    INTERNAL,
    BRIDGE,
    MACVLAN,
    IPVLAN,
    UNKNOWN,
};

static const auto strings = std::map<network_type_t, std::string>{
    {network_type_t::NONE, "none"},
    {network_type_t::INTERNAL, "internal"},
    {network_type_t::BRIDGE, "bridge"},
    {network_type_t::MACVLAN, "macvlan"},
    {network_type_t::IPVLAN, "ipvlan"},
};

inline auto to_string(const network_type_t& network_type) //
    -> std::string
{
    return strings.count(network_type) ? strings.at(network_type) : "unknown";
}

inline auto network_type_from_string(std::string_view str) //
    -> network_type_t
{
    for (const auto& it : strings)
    {
        if (it.second == str)
        {
            return it.first;
        }
    }
    return network_type_t::UNKNOWN;
}

} // namespace FLECS

#endif /* FF5A0392_A69F_47AE_9AD0_EE3218DC149C */
