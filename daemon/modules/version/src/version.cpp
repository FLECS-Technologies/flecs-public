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

#include "version.h"

#include <factory/factory.h>

#include <cstdio>

namespace FLECS {

namespace {
register_module_t<module_version_t> _reg("version");
}

module_version_t::module_version_t()
{}

auto module_version_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/system/version").methods("GET"_method)([=, this]() { return http_version(); });
}

auto module_version_t::do_deinit() //
    -> void
{}

auto module_version_t::http_version() const //
    -> crow::response
{
    using std::operator""s;

    auto response = json_t{};
    response["core"] = core_version();

    return crow::response{crow::status::OK, "json", response.dump()};
}

auto module_version_t::core_version() const //
    -> std::string
{
    return std::string{FLECS_VERSION} + "-" + FLECS_GIT_SHA;
}

auto module_version_t::api_version() const //
    -> std::string
{
    return FLECS_API_VERSION;
}

} // namespace FLECS
