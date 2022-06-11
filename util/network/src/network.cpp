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

#include <arpa/inet.h>

#include <bitset>
#include <regex>

#include "util/string/string_utils.h"

namespace FLECS {

auto ipv4_to_bits(std::string_view ip) //
    -> in_addr
{
    auto ip_addr = in_addr{};
    inet_pton(AF_INET, ip.data(), &ip_addr);
    return ip_addr;
}

auto ipv6_to_bits(std::string_view ip) //
    -> in6_addr
{
    auto ip_addr = in6_addr{};
    inet_pton(AF_INET6, ip.data(), &ip_addr);
    return ip_addr;
}

auto ipv4_to_string(const in_addr& ip) //
    -> std::string
{
    char buf[INET_ADDRSTRLEN] = {};
    inet_ntop(AF_INET, &ip, buf, INET_ADDRSTRLEN);
    return std::string{buf};
}

auto ipv6_to_string(const in6_addr& ip) //
    -> std::string
{
    char buf[INET6_ADDRSTRLEN] = {};
    inet_ntop(AF_INET6, &ip, buf, INET6_ADDRSTRLEN);
    return std::string{buf};
}

auto subnet_mask_to_cidr_v4(std::string_view subnet_mask) //
    -> std::size_t
{
    const auto subnet_bits = std::bitset<8 * sizeof(in_addr_t)>{ipv4_to_bits(subnet_mask).s_addr};
    return subnet_bits.count();
}

auto cidr_to_subnet_mask_v4(std::string_view cidr_subnet) //
    -> std::string
{
    // until end of line: d/[0-32]
    // e.g. 127.0.0.1/24 -> 1/24
    const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
    auto m = std::cmatch{};
    if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2)
    {
        return std::string{};
    }

    const auto subnet_size = std::stoi(m[1].str());
    auto subnet_bits = std::bitset<8 * sizeof(in_addr_t)>{};
    for (auto i = 0; i < subnet_size; ++i)
    {
        subnet_bits.set(subnet_bits.size() - i - 1);
    }

    const auto addr = in_addr{.s_addr = static_cast<in_addr_t>(subnet_bits.to_ulong())};

    return ipv4_to_string(addr);
}

auto ipv4_to_network(std::string_view ip, std::string_view subnet_mask) //
    -> std::string
{
    const auto ip_addr = ipv4_to_bits(ip).s_addr;
    const auto subnet_addr = ipv4_to_bits(subnet_mask).s_addr;

    const auto network_addr = in_addr{.s_addr = (ip_addr & subnet_addr)};

    return ipv4_to_string(network_addr) + "/" + stringify(subnet_mask_to_cidr_v4(subnet_mask));
}

} // namespace FLECS
