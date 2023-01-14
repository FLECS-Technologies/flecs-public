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

#include "ip_addr.h"

#include <arpa/inet.h>

#include <cmath>
#include <stdexcept>

namespace FLECS {

ip_addr_t::ip_addr_t()
    : _addr{}
{}

ip_addr_t::ip_addr_t(in_addr ip)
    : _addr{std::move(ip)}
{}

ip_addr_t::ip_addr_t(in_addr_t ip)
    : _addr{in_addr{.s_addr = ip}}
{}

ip_addr_t::ip_addr_t(in6_addr ip)
    : _addr{std::move(ip)}
{}

ip_addr_t::ip_addr_t(std::string_view addr)
    : _addr{}
{
    this->addr(std::move(addr));
}

/** @todo */
template <class... Ts>
struct overload : Ts...
{
    using Ts::operator()...;
};
template <class... Ts>
overload(Ts...) -> overload<Ts...>;

auto ip_addr_t::type() const noexcept //
    -> type_e
{
    return std::visit(
        overload{
            [](const std::monostate&) { return type_e::None; },
            [](const ipv4_t&) { return type_e::IPv4; },
            [](const ipv6_t&) { return type_e::IPv6; }},
        _addr);
}

auto ip_addr_t::addr(std::string_view addr) //
    -> void
{
    /* try to parse as IPv4 */
    auto ipv4 = in_addr{};
    if (inet_pton(AF_INET, addr.data(), &ipv4) == 1) {
        _addr = std::move(ipv4);
        return;
    }
    /* try to parse as IPv6 */
    auto ipv6 = in6_addr{};
    if (inet_pton(AF_INET6, addr.data(), &ipv6) == 1) {
        _addr = std::move(ipv6);
        return;
    }
    _addr = none_t{};
}

auto ip_addr_t::addr(in_addr addr) noexcept //
    -> void
{
    _addr = std::move(addr);
}

auto ip_addr_t::addr(in_addr_t addr) noexcept //
    -> void
{
    _addr = in_addr{.s_addr = addr};
}

auto ip_addr_t::addr(in6_addr addr) noexcept //
    -> void
{
    _addr = std::move(addr);
}

auto ip_addr_t::addr_v4() const //
    -> const in_addr&
{
    return std::get<ipv4_t>(_addr);
}

auto ip_addr_t::addr_v6() const //
    -> const in6_addr&
{
    return std::get<ipv6_t>(_addr);
}

auto ip_addr_t::operator++() //
    -> ip_addr_t&
{
    *this += 1;
    return *this;
}

auto ip_addr_t::operator++(int) //
    -> ip_addr_t
{
    auto temp = *this;
    *this += 1;
    return temp;
}

auto ip_addr_t::operator--() //
    -> ip_addr_t&
{
    return *this -= 1;
}

auto ip_addr_t::operator--(int) //
    -> ip_addr_t
{
    auto temp = *this;
    *this -= 1;
    return temp;
}

auto ip_addr_t::operator+=(std::int64_t n) //
    -> ip_addr_t&
{
    std::visit(
        overload{
            [](const none_t&) {},
            [&n](ipv4_t& addr) { addr.s_addr = htonl(ntohl(addr.s_addr) + n); },
            [&n](ipv6_t& addr) {
                std::uint64_t upper = static_cast<std::uint64_t>(ntohl(addr.s6_addr32[0])) << 32 |
                                      ntohl(addr.s6_addr32[1]);
                std::uint64_t lower = static_cast<std::uint64_t>(ntohl(addr.s6_addr32[2])) << 32 |
                                      ntohl(addr.s6_addr32[3]);
                auto new_lower = lower + n;
                if ((n > 0) && (new_lower <= lower)) {
                    ++upper;
                } else if ((n < 0) && (new_lower >= lower)) {
                    --upper;
                }
                addr.s6_addr32[0] = htonl(upper >> 32);
                addr.s6_addr32[1] = htonl(upper & 0xFFFFFFFF);
                addr.s6_addr32[2] = htonl(new_lower >> 32);
                addr.s6_addr32[3] = htonl(new_lower & 0xFFFFFFFF);
            }},
        _addr);

    return *this;
}

auto ip_addr_t::operator-=(std::int64_t n) //
    -> ip_addr_t&
{
    *this += (-n);
    return *this;
}

auto ip_addr_t::operator+(std::int64_t n) //
    -> ip_addr_t
{
    auto temp = *this;
    temp += n;
    return temp;
}

auto ip_addr_t::operator-(std::int64_t n) //
    -> ip_addr_t
{
    auto temp = *this;
    temp -= n;
    return temp;
}

bool operator<(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    if (lhs.type() != rhs.type()) {
        throw std::runtime_error{"Cannot compare IP addresses of different type"};
    }

    return std::visit(
        overload{
            [](const ip_addr_t::none_t&) {
                throw std::runtime_error{"Cannot compare uninitialized IP addresses"};
                return false;
            },
            [&lhs, &rhs](const ip_addr_t::ipv4_t&) {
                return ntohl(lhs.addr_v4().s_addr) < ntohl(rhs.addr_v4().s_addr);
            },
            [&lhs, &rhs](const ip_addr_t::ipv6_t&) {
                return std::forward_as_tuple(
                           ntohl(lhs.addr_v6().s6_addr32[0]),
                           ntohl(lhs.addr_v6().s6_addr32[1]),
                           ntohl(lhs.addr_v6().s6_addr32[2]),
                           ntohl(lhs.addr_v6().s6_addr32[3])) //
                       < std::forward_as_tuple(
                             ntohl(rhs.addr_v6().s6_addr32[0]),
                             ntohl(rhs.addr_v6().s6_addr32[1]),
                             ntohl(rhs.addr_v6().s6_addr32[2]),
                             ntohl(rhs.addr_v6().s6_addr32[3]));
            }},
        lhs._addr);
}

bool operator<=(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    return !(lhs > rhs);
}

bool operator>(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    return rhs < lhs;
}

bool operator>=(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    return !(lhs < rhs);
}

bool operator==(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    if (lhs.type() != rhs.type()) {
        return false;
    }

    return std::visit(
        overload{
            [](const ip_addr_t::none_t&) { return true; },
            [&rhs](const ip_addr_t::ipv4_t& addr) { return addr.s_addr == rhs.addr_v4().s_addr; },
            [&rhs](const ip_addr_t::ipv6_t& addr) {
                return std::equal(
                    addr.s6_addr,
                    addr.s6_addr + sizeof(addr.__in6_u),
                    rhs.addr_v6().s6_addr);
            }},
        lhs._addr);
}

bool operator!=(const ip_addr_t& lhs, const ip_addr_t& rhs)
{
    return !(lhs == rhs);
}

auto to_string(const ip_addr_t& addr) //
    -> std::string
{
    return std::visit(
        overload{
            [](const ip_addr_t::none_t&) { return std::string{}; },
            [](const ip_addr_t::ipv4_t& addr) {
                char buf[INET_ADDRSTRLEN] = {};
                inet_ntop(AF_INET, &addr, buf, INET_ADDRSTRLEN);
                return std::string{buf};
            },
            [](const ip_addr_t::ipv6_t& addr) {
                char buf[INET6_ADDRSTRLEN] = {};
                inet_ntop(AF_INET6, &addr, buf, INET6_ADDRSTRLEN);
                return std::string{buf};
            }},
        addr._addr);
}

} // namespace FLECS
