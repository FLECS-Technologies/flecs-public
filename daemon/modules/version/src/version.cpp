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

#include "version.h"

#include <factory/factory.h>

#include <cstdio>

namespace FLECS {

namespace {
register_module_t<module_version_t> _reg("version");
}

module_version_t::module_version_t()
{}

auto module_version_t::version(json_t& response) const //
    -> crow::response
{
    response["core"] = std::string{FLECS_VERSION} + "-" + std::string{FLECS_GIT_SHA};

    return crow::response{crow::status::OK, response.dump()};
}

auto module_version_t::do_init() //
    -> void
{
    FLECS_ROUTE("/system/version").methods("GET"_method)([=]() {
        auto response = json_t{};
        return version(response);
    });
}

} // namespace FLECS
