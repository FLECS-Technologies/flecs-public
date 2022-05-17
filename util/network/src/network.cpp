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

#include "network.h"

#include <bitset>

#include "util/string/string_utils.h"

namespace FLECS {

in_addr ipv4_to_bits(const std::string& ip)
{
    auto ip_addr = in_addr{};
    inet_pton(AF_INET, ip.c_str(), &ip_addr);
    return ip_addr;
}

in6_addr ipv6_to_bits(const std::string& ip)
{
    auto ip_addr = in6_addr{};
    inet_pton(AF_INET6, ip.c_str(), &ip_addr);
    return ip_addr;
}

std::string ipv4_to_string(const in_addr& ip)
{
    char buf[INET_ADDRSTRLEN] = {};
    inet_ntop(AF_INET, &ip, buf, INET_ADDRSTRLEN);
    return std::string{buf};
}

std::string ipv6_to_string(const in6_addr& ip)
{
    char buf[INET6_ADDRSTRLEN] = {};
    inet_ntop(AF_INET6, &ip, buf, INET6_ADDRSTRLEN);
    return std::string{buf};
}

std::size_t subnet_to_cidr_v4(const std::string& subnet_mask)
{
    auto subnet_bits = std::bitset<8 * sizeof(in_addr_t)>{ipv4_to_bits(subnet_mask).s_addr};
    return subnet_bits.count();
}

std::string ipv4_to_network(const std::string& ip, const std::string& subnet_mask)
{
    const auto ip_addr = ipv4_to_bits(ip).s_addr;
    const auto subnet_addr = ipv4_to_bits(subnet_mask).s_addr;

    const auto network_addr = in_addr{.s_addr = (ip_addr & subnet_addr)};

    return ipv4_to_string(network_addr) + "/" + stringify(subnet_to_cidr_v4(subnet_mask));
}

} // namespace FLECS
