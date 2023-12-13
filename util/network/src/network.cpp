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

#include "util/network/network.h"

#include <arpa/inet.h>

#include <bitset>
#include <regex>

#include "util/network/ip_addr.h"
#include "util/string/string_utils.h"

namespace flecs {

auto subnet_mask_to_cidr_v4(std::string_view subnet_mask) //
    -> std::size_t
{
    const auto addr = ip_addr_t{subnet_mask};
    if (addr.type() != ip_addr_t::IPv4) {
        return {};
    }
    const auto subnet_bits = std::bitset<8 * sizeof(in_addr_t)>{addr.addr_v4().s_addr};
    return subnet_bits.count();
}

auto cidr_to_subnet_mask_v4(std::string_view cidr_subnet) //
    -> std::string
{
    // until end of line: d/[0-32]
    // e.g. 127.0.0.1/24 -> 1/24
    const auto subnet_regex = std::regex{R"(\d\/([0-9]|[1][0-9]|[2][0-9]|[3][0-2])$)"};
    auto m = std::cmatch{};
    if (!std::regex_search(cidr_subnet.data(), m, subnet_regex) || m.size() < 2) {
        return std::string{};
    }

    const auto subnet_size = std::stoi(m[1].str());
    auto subnet_bits = std::bitset<8 * sizeof(in_addr_t)>{};
    for (auto i = 0; i < subnet_size; ++i) {
        subnet_bits.set(subnet_bits.size() - i - 1);
    }

    const auto addr = ip_addr_t{htonl(static_cast<in_addr_t>(subnet_bits.to_ulong()))};
    return to_string(addr);
}

auto ipv4_to_network(std::string_view ip, std::string_view subnet_mask) //
    -> std::string
{
    const auto ip_addr = ip_addr_t{ip};
    const auto subnet_addr = subnet_mask_t{subnet_mask};
    if (ip_addr.type() != ip_addr_t::IPv4 || subnet_addr.type() != ip_addr_t::IPv4) {
        return {};
    }

    const auto network_addr = ip_addr_t{ip_addr.addr_v4().s_addr & subnet_addr.addr_v4().s_addr};
    return stringify_delim('/', network_addr, subnet_mask_to_cidr_v4(subnet_mask));
}

} // namespace flecs
