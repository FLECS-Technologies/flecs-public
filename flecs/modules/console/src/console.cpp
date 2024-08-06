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

#include "flecs/modules/console/console.h"

#include "flecs/modules/console/impl/console_impl.h"
#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {


console_t::console_t()
    : _impl{std::make_unique<impl::console_t>(this)}
{}

console_t::~console_t() = default;

auto console_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/console/authentication").methods("PUT"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON(args, auth, console::auth_response_data_t);

        return store_authentication(auth);
    });

    FLECS_V2_ROUTE("/console/authentication")
        .methods("DELETE"_method)([this](const crow::request& /* req */) { return delete_authentication(); });

    return _impl->do_init();
}

auto console_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto console_t::authentication() const noexcept //
    -> const console::auth_response_data_t&
{
    return _impl->do_authentication();
}

auto console_t::activate_license(std::string license, const std::optional<console::session_id_t>& session_id) //
    -> license_activation_result_t
{
    return _impl->do_activate_license(license, session_id);
}

auto console_t::activate_license_key() //
    -> license_activation_result_t
{
    return _impl->do_activate_license_key();
}

auto console_t::validate_license(std::string_view session_id) //
    -> result_t
{
    return _impl->do_validate_license(session_id);
}

auto console_t::acquire_download_token(std::string app, std::string version, std::string session_id) //
    -> std::optional<console::download_token_t>
{
    return _impl->do_acquire_download_token(app, version, session_id);
}

auto console_t::store_authentication(console::auth_response_data_t auth) //
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
