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

#include "daemon/modules/console/console.h"

#include "daemon/modules/console/impl/console_impl.h"
#include "factory/factory.h"

namespace flecs {
namespace module {

namespace {
register_module_t<console_t> _reg("console");
}

console_t::console_t()
    : _impl{std::make_unique<impl::console_t>()}
{}

console_t::~console_t() = default;

auto console_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/console/authentication")
        .methods("PUT"_method)([this](const crow::request& req) {
            auto response = json_t{};
            const auto args = parse_json(req.body);
            REQUIRED_TYPED_JSON(args, auth, console::auth_response_t);

            return store_authentication(auth);
        });

    FLECS_V2_ROUTE("/console/authentication")
        .methods("DELETE"_method)(
            [this](const crow::request& /* req */) { return delete_authentication(); });
}

auto console_t::do_deinit() //
    -> void
{}

auto console_t::authentication() const noexcept //
    -> const console::auth_response_t&
{
    return _impl->do_authentication();
}

auto console_t::activate_license(std::string_view session_id) //
    -> result_t
{
    return _impl->do_activate_license(session_id);
}

auto console_t::validate_license(std::string_view session_id) //
    -> result_t
{
    return _impl->do_validate_license(session_id);
}

auto console_t::store_authentication(console::auth_response_t auth) //
    -> crow::response
{
    return _impl->do_store_authentication(auth);
}

auto console_t::delete_authentication() //
    -> crow::response
{
    return _impl->do_delete_authentication();
}

} // namespace module
} // namespace flecs
