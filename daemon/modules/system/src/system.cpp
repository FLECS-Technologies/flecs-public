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

#include "system.h"

#include <ifaddrs.h>
#include <linux/if_link.h>
#include <netpacket/packet.h>
#include <sys/types.h>

#include <cstdio>
#include <fstream>
#include <limits>
#include <sstream>

#include "factory/factory.h"
#include "util/cxx20/string.h"
#include "util/network/network.h"
#include "util/string/string_utils.h"

namespace FLECS {

namespace {
register_module_t<module_system_t> _reg("system");
}

module_system_t::module_system_t()
{}

auto module_system_t::do_init() //
    -> void
{
    FLECS_ROUTE("/system/ping").methods("GET"_method)([=]() {
        auto response = json_t{};
        return ping(response);
    });
}

auto module_system_t::ping(json_t &response) const //
    -> crow::response
{
    response["additionalInfo"] = "OK";

    return crow::response{crow::status::OK, response.dump()};
}

auto module_system_t::get_network_adapters() const -> std::map<std::string, netif_t>
{
    auto adapters = std::map<std::string, netif_t>{};

    auto ifa = (ifaddrs *){};
    const auto res = getifaddrs(&ifa);
    if (res != 0)
    {
        return adapters;
    }

    const auto root = ifa;

    while (ifa)
    {
        if (!ifa->ifa_addr)
        {
            ifa = ifa->ifa_next;
            continue;
        }
        switch (ifa->ifa_addr->sa_family)
        {
            case AF_PACKET: {
                char buf[18];
                struct sockaddr_ll *s = (struct sockaddr_ll *)ifa->ifa_addr;
                sprintf(
                    buf,
                    "%02X:%02X:%02X:%02X:%02X:%02X",
                    s->sll_addr[0],
                    s->sll_addr[1],
                    s->sll_addr[2],
                    s->sll_addr[3],
                    s->sll_addr[4],
                    s->sll_addr[5]);
                adapters[ifa->ifa_name].mac = buf;
                break;
            }
            case AF_INET: {
                auto ip = ipaddr_t{};

                // inet_ntop(AF_INET, &((struct sockaddr_in *)ifa->ifa_netmask)->sin_addr, buf, INET_ADDRSTRLEN);
                ip.addr = ipv4_to_string(((struct sockaddr_in *)ifa->ifa_addr)->sin_addr);
                ip.subnet_mask = ipv4_to_string(((struct sockaddr_in *)ifa->ifa_netmask)->sin_addr);

                adapters[ifa->ifa_name].ipv4_addr.emplace_back(ip);
                break;
            }
            case AF_INET6: {
                auto ip = ipaddr_t{};

                ip.addr = ipv6_to_string(((struct sockaddr_in6 *)ifa->ifa_addr)->sin6_addr);
                ip.subnet_mask = ipv6_to_string(((struct sockaddr_in6 *)ifa->ifa_netmask)->sin6_addr);

                adapters[ifa->ifa_name].ipv6_addr.emplace_back(ip);
                break;
            }
            default: {
                break;
            }
        }
        ifa = ifa->ifa_next;
    }

    freeifaddrs(root);

    auto route_file = std::ifstream{"/proc/net/route"};
    route_file.ignore(std::numeric_limits<std::streamsize>::max(), route_file.widen('\n'));
    auto line = std::string{};
    while (std::getline(route_file, line))
    {
        enum route_columns_t
        {
            IFACE,
            DESTINATION,
            GATEWAY,
            FLAGS,
            REFCNT,
            USE,
            METRIC,
            MASK,
            MTU,
            WINDOW,
            IRTT,
            ROUTE_COLUMNS_COUNT,
        };
        const auto parts = split(line, '\t');
        if (parts.size() != ROUTE_COLUMNS_COUNT)
        {
            continue;
        }
        auto sstream = std::stringstream{};
        auto destination = std::int32_t{};
        sstream << parts[DESTINATION];
        sstream >> std::hex >> destination;
        if (destination == 0)
        {
            auto sstream = std::stringstream{};
            auto gateway = std::int32_t{};
            sstream << parts[GATEWAY];
            sstream >> std::hex >> gateway;
            char buf[INET_ADDRSTRLEN] = {};
            inet_ntop(AF_INET, &gateway, buf, INET_ADDRSTRLEN);
            adapters[parts[IFACE]].gateway = buf;
        }
    }

    for (decltype(auto) adapter : adapters)
    {
        if ((cxx20::starts_with(adapter.first, "en") || (cxx20::starts_with(adapter.first, "eth"))))
        {
            adapter.second.type = netif_type_t::WIRED;
        }
        else if ((cxx20::starts_with(adapter.first, "wl")))
        {
            adapter.second.type = netif_type_t::WIRELESS;
        }
        else if ((cxx20::starts_with(adapter.first, "lo")))
        {
            adapter.second.type = netif_type_t::LOCAL;
        }
        else if ((cxx20::starts_with(adapter.first, "veth")))
        {
            adapter.second.type = netif_type_t::VIRTUAL;
        }
        else if ((cxx20::starts_with(adapter.first, "br") || (cxx20::starts_with(adapter.first, "docker"))))
        {
            adapter.second.type = netif_type_t::BRIDGE;
        }
        else
        {
            adapter.second.type = UNKNOWN;
        }
    }

    return adapters;
}

} // namespace FLECS
