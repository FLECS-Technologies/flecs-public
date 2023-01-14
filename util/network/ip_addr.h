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

#include <netinet/in.h>

#include <string>
#include <string_view>
#include <tuple>
#include <variant>

namespace FLECS {

class ip_addr_t
{
public:
    enum type_e {
        None,
        IPv4,
        IPv6,
    };

    ip_addr_t();
    ip_addr_t(in_addr ip);
    ip_addr_t(in_addr_t ip);
    ip_addr_t(in6_addr ip);
    ip_addr_t(std::string_view addr);

    auto type() const noexcept //
        -> type_e;

    auto addr(std::string_view addr) //
        -> void;
    auto addr(in_addr addr) noexcept //
        -> void;
    auto addr(in_addr_t addr) noexcept //
        -> void;
    auto addr(in6_addr addr) noexcept //
        -> void;

    auto addr_v4() const //
        -> const in_addr&;
    auto addr_v6() const //
        -> const in6_addr&;

    auto operator++() //
        -> ip_addr_t&;
    auto operator++(int) //
        -> ip_addr_t;
    auto operator--() //
        -> ip_addr_t&;
    auto operator--(int) //
        -> ip_addr_t;

    auto operator+=(std::int64_t) //
        -> ip_addr_t&;
    auto operator-=(std::int64_t) //
        -> ip_addr_t&;

    auto operator+(std::int64_t) //
        -> ip_addr_t;
    auto operator-(std::int64_t) //
        -> ip_addr_t;

private:
    friend auto operator<(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator<(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator<=(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator>(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator>=(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator==(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;
    friend auto operator!=(const ip_addr_t& lhs, const ip_addr_t& rhs) //
        -> bool;

    friend auto to_string(const ip_addr_t& addr) //
        -> std::string;

    using none_t = std::monostate;
    using ipv4_t = in_addr;
    using ipv6_t = in6_addr;

    std::variant<none_t, ipv4_t, ipv6_t> _addr;
};

using subnet_mask_t = ip_addr_t;

} // namespace FLECS
