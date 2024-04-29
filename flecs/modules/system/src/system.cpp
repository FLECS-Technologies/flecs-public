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

#include "flecs/modules/system/system.h"

#include <ifaddrs.h>
#include <linux/if_link.h>
#include <netpacket/packet.h>
#include <sys/types.h>

#include <cstdio>
#include <fstream>
#include <limits>
#include <sstream>

#include "flecs/modules/factory/factory.h"
#include "flecs/util/network/ip_addr.h"
#include "flecs/util/network/network.h"
#include "flecs/util/string/string_utils.h"
#include "flecs/util/sysinfo/sysinfo.h"

namespace flecs {
namespace module {

namespace {
register_module_t<system_t> _reg("system");
}

system_t::system_t()
{}

auto system_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/system/ping").methods("GET"_method)([this]() { return ping(); });

    FLECS_V2_ROUTE("/system/info").methods("GET"_method)([this]() { return info(); });
}

auto system_t::do_deinit() //
    -> void
{}

auto system_t::ping() const //
    -> crow::response
{
    const auto response = json_t({
        {"additionalInfo", "OK"},
    });

    return crow::response{crow::status::OK, "json", response.dump()};
}

auto system_t::info() const //
    -> crow::response
{
    const auto response = json_t(sysinfo_t{});

    return crow::response{crow::status::OK, "json", response.dump()};
}

auto system_t::get_network_adapters() const //
    -> std::map<std::string, NetInfo>
{
    auto adapters = std::map<std::string, NetInfo>{};
    for (auto adapter : read_network_adapters()) {
        adapters.insert({std::string(adapter.name.c_str()), std::move(adapter.info)});
    }
    return adapters;
}

} // namespace module
} // namespace flecs
