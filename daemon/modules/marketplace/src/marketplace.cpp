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

#include "marketplace.h"

#include "factory/factory.h"

namespace FLECS {

namespace {
register_module_t<module_marketplace_t> _reg("mp");
}

module_marketplace_t::module_marketplace_t()
{}

auto module_marketplace_t::do_init() //
    -> void
{
    FLECS_ROUTE("/marketplace/login").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_JSON_VALUE(args, user);
        REQUIRED_JSON_VALUE(args, token);

        return login(user, token);
    });

    FLECS_ROUTE("/marketplace/logout").methods("POST"_method)([=](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        OPTIONAL_JSON_VALUE(args, user);

        return logout(user);
    });
}

auto module_marketplace_t::login(std::string user, std::string token) //
    -> crow::response
{
    _user = std::move(user);
    _token = std::move(token);

    const auto response = json_t({
        {"additionalInfo", "OK"},
    });
    return crow::response{crow::status::OK, "json", response.dump()};
}

auto module_marketplace_t::logout(std::string_view /*user*/) //
    -> crow::response
{
    _user.clear();
    _token.clear();

    const auto response = json_t({
        {"additionalInfo", "OK"},
    });
    return crow::response{crow::status::OK, "json", response.dump()};
}

} // namespace FLECS
