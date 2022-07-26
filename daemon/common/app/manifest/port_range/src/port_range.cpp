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

#include "port_range.h"

#include <algorithm>

#include "util/string/string_utils.h"

namespace FLECS {

port_t::port_t(const std::string& port_str) noexcept
    : _port{}
{
    try
    {
        // convert string to int
        auto pos = size_t{};
        const auto port = std::stoi(port_str, &pos);
        // ensure whole string was matched, ignore "0x9000" or "9000andsomethingelse"
        if (pos != port_str.length())
        {
            _port = static_cast<value_t>(0);
        }
        else if (port > std::numeric_limits<value_t>::min() && port <= std::numeric_limits<value_t>::max())
        {
            _port = static_cast<value_t>(port);
        }
    }
    catch (const std::exception&)
    {
        _port = static_cast<value_t>(0);
    }
}

port_range_t::port_range_t(const std::string& range_str) noexcept
    : _start_port{0}
    , _end_port{0}
{
    const auto range = split(range_str, '-');
    // - 9000
    if (range.size() == 1)
    {
        _start_port = port_t{range[0]};
        _end_port = _start_port;
    }
    // - 9000-9005
    else if (range.size() == 2)
    {
        _start_port = port_t{range[0]};
        _end_port = port_t{range[1]};
    }
}

mapped_port_range_t::mapped_port_range_t(const std::string& map_str)
{
    const auto ranges = split(map_str, ':');
    if (ranges.size() == 1)
    {
        // use same port for host and container, i.e.:
        // - 9000       # map host-port 9000 to container-port 9000
        // - 9000-9005  # map host-port 9000 to container-port 9005
        const auto port_range = port_range_t{ranges[0]};
        if (port_range.is_valid())
        {
            _host_port_range = port_range;
            _container_port_range = port_range;
        }
    }
    else if (ranges.size() == 2)
    {
        // use different port for host and container, i.e.
        // - 9000:9001              # map host-port 9000 to container-port 9001
        // - 9000-9005:9001-9006    # map host-ports 9000-9005 to container-ports 9001-9006
        const auto host_range = port_range_t{ranges[0]};
        const auto container_range = port_range_t{ranges[1]};
        if (host_range.is_valid() && container_range.is_valid())
        {
            _host_port_range = host_range;
            _container_port_range = container_range;
        }
        // - :9001-9006             # map random host-ports to container-ports 9001-9006
        else if (
            container_range.is_valid() &&
            (host_range.start_port() == 0 && host_range.end_port() == 0 && ranges[0].empty()))
        {
            _host_port_range = invalid_port_range;
            _container_port_range = container_range;
        }
    }
}

auto to_json(json_t& json, const mapped_port_range_t& mapped_port_range) //
    -> void
{
    json = json_t{
        {"container", stringify(mapped_port_range.container_port_range())},
        {"host", stringify(mapped_port_range.host_port_range())},
    };
}

auto from_json(const json_t& json, mapped_port_range_t& mapped_port_range) //
    -> void
{
    auto host = std::string{};
    auto container = std::string{};

    json.at("host").get_to(host);
    json.at("container").get_to(container);

    mapped_port_range = mapped_port_range_t{host + ":" + container};
}

} // namespace FLECS
