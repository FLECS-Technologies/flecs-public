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

#include <cstdint>
#include <limits>
#include <string>
#include <type_traits>

#include "flecs/util/json/json.h"
#include "flecs/util/string/string_utils.h"

namespace flecs {

class port_t
{
public:
    using value_t = std::uint16_t;

    constexpr port_t() noexcept
        : _port{}
    {}

    explicit port_t(const std::string& port_str) noexcept;

    constexpr port_t(value_t port) noexcept
        : _port{port}
    {}

    constexpr bool is_valid() const noexcept { return _port != 0; }

    constexpr operator value_t() const noexcept { return _port; }

private:
    value_t _port;
};

inline std::string to_string(const port_t& port_t)
{
    return stringify(static_cast<port_t::value_t>(port_t));
}

class port_range_t
{
public:
    constexpr port_range_t() noexcept
        : _start_port{}
        , _end_port{}
    {}

    explicit constexpr port_range_t(port_t port) noexcept
        : _start_port{port}
        , _end_port{port}
    {}

    constexpr port_range_t(port_t start_port, port_t end_port) noexcept
        : _start_port{start_port}
        , _end_port{end_port}
    {}

    port_range_t(const std::string& range_str) noexcept;

    constexpr bool is_valid() const noexcept { return (_start_port.is_valid()) && (_end_port.is_valid()); }

    constexpr port_t start_port() const noexcept { return _start_port; }
    constexpr port_t end_port() const noexcept { return _end_port; }
    constexpr bool does_collide_with(const port_range_t& other) const
    {
        return _start_port <= other.end_port() && other._start_port <= _end_port;
    }

private:
    friend auto operator<=>(const port_range_t&, const port_range_t&) = default;

    port_t _start_port;
    port_t _end_port;
};

constexpr auto invalid_port_range = port_range_t{port_t{0}, port_t{0}};

inline std::string to_string(const port_range_t& port_range)
{
    auto res = stringify(port_range.start_port());

    if (port_range.start_port() != port_range.end_port()) {
        res.append("-" + stringify(port_range.end_port()));
    }

    return res;
}

class mapped_port_range_t
{
public:
    constexpr mapped_port_range_t() noexcept
        : _host_port_range{}
        , _container_port_range{}
    {}

    constexpr explicit mapped_port_range_t(port_range_t host_port_range) noexcept
        : _host_port_range{host_port_range}
        , _container_port_range{host_port_range}
    {}

    constexpr mapped_port_range_t(port_range_t host_port_range, port_range_t container_port_range) noexcept
        : _host_port_range{host_port_range}
        , _container_port_range{container_port_range}
    {}

    mapped_port_range_t(std::string_view map_str);

    constexpr bool is_valid() const noexcept
    {
        // a mapped range may have {0,0} as host_port_range, indicating randomization of host ports
        const auto host_random = (_host_port_range.start_port() == 0 && _host_port_range.end_port() == 0);
        const auto host_valid = host_random || _host_port_range.is_valid();

        const auto container_valid = _container_port_range.is_valid();

        const auto ranges_valid =
            host_random || ((_container_port_range.end_port() - _container_port_range.start_port()) ==
                            (_host_port_range.end_port() - _host_port_range.start_port()));

        return host_valid && container_valid && ranges_valid;
    }

    constexpr port_range_t host_port_range() const noexcept { return _host_port_range; }

    constexpr port_range_t container_port_range() const noexcept { return _container_port_range; }

private:
    friend auto operator<=>(const mapped_port_range_t&, const mapped_port_range_t&) = default;

    friend auto to_json(json_t& json, const mapped_port_range_t& mapped_port_range) //
        -> void;
    friend auto from_json(const json_t& json, mapped_port_range_t& mapped_port_range) //
        -> void;

    port_range_t _host_port_range;
    port_range_t _container_port_range;
};

inline std::string to_string(const mapped_port_range_t& mapped_port_range)
{
    auto res = std::string{};
    if (mapped_port_range.host_port_range().is_valid()) {
        res += stringify(mapped_port_range.host_port_range());
    }
    res += ":";

    res += stringify(mapped_port_range.container_port_range());

    return res;
}

} // namespace flecs
