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

#include "deployment.h"

#include <cassert>
#include <map>
#include <regex>
#include <set>

#include "util/network/network.h"

namespace FLECS {

auto to_string(const network_type_t& network_type) //
    -> std::string
{
    const auto strings = std::map<network_type_t, std::string>{
        {network_type_t::BRIDGE, "bridge"},
        {network_type_t::MACVLAN, "macvlan"},
        {network_type_t::IPVLAN, "ipvlan"},
    };

    return strings.at(network_type);
}

auto network_type_from_string(std::string_view str) //
    -> network_type_t
{
    const auto types = std::map<std::string_view, network_type_t>{
        {"default", network_type_t::BRIDGE},
        {"bridge", network_type_t::BRIDGE},
        {"ipvlan", network_type_t::IPVLAN},
        {"macvlan", network_type_t::MACVLAN},
    };

    return types.at(str);
}

auto deployment_t::generate_instance_ip(std::string_view cidr_subnet, std::string_view gateway) const //
    -> std::string
{
    // parse a.b.c.d
    auto base_ip = in_addr_t{};
    {
        // from beginning of line: (a.b.c.)(d/)
        // e.g. 127.0.0.1/24 -> (127.0.0.)(1)/
        const auto ip_regex = std::regex{R"((^(?:\d{1,3}\.){3}\d{1,3})\/)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, ip_regex))
        {
            return std::string{};
        }
        base_ip = ntohl(ipv4_to_bits(m[1].str()).s_addr);
    }
    // parse /x
    auto subnet_size = int{};
    {
        // until end of line: d/[0-32]
        // e.g. 127.0.0.1/24 -> 1/24
        const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
        auto m = std::cmatch{};
        if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2)
        {
            return std::string{};
        }
        subnet_size = std::stoi(m[1].str());
    }

    // determine the last ip address that belongs to the subnet:
    // (~0u << subnet_size) creates an inverted bitmask (such as 0xff00),
    // which is again inverted (yielding e.g. 0x00ff) and or'ed with the base ip.
    // finally subtract 1 to exclude the network's broadcast address
    const auto max_ip = (base_ip | ~(~0u << subnet_size)) - 1;

    auto used_ips = std::set<in_addr_t>{};
    if (!gateway.empty())
    {
        used_ips.emplace(ntohl(ipv4_to_bits(gateway).s_addr));
    }
    for (const auto& instance : _instances)
    {
        for (const auto& network : instance.second.config().networks)
        {
            used_ips.emplace(ntohl(ipv4_to_bits(network.ip).s_addr));
        }
    }

    // skip network address and host address
    auto instance_ip = base_ip + 2;

    // search first unused address
    while (used_ips.find(instance_ip) != used_ips.end())
    {
        ++instance_ip;
    }

    if (instance_ip > max_ip)
    {
        return std::string{};
    }

    return ipv4_to_string(in_addr{.s_addr = htonl(instance_ip)});
}

} // namespace FLECS
