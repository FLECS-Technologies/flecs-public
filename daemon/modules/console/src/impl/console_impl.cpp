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

#include "daemon/modules/console/impl/console_impl.h"

#include "util/json/json.h"

namespace flecs {
namespace module {
namespace impl {

console_t::console_t()
{}

console_t::~console_t()
{}

auto console_t::do_init() //
    -> void
{}

auto console_t::do_deinit() //
    -> void
{}

auto console_t::do_authentication() const noexcept //
    -> const console::auth_response_data_t&
{
    return _auth;
}

auto console_t::do_activate_license(std::string_view /*session_id*/) //
    -> result_t
{
    return {0, {}};
}

auto console_t::do_validate_license(std::string_view /*session_id*/) //
    -> result_t
{
    return {0, {}};
}

auto console_t::do_store_authentication(console::auth_response_data_t auth) //
    -> crow::response
{
    _auth = std::move(auth);

    return crow::response{crow::NO_CONTENT};
}

auto console_t::do_delete_authentication() //
    -> crow::response
{
    _auth = console::auth_response_data_t{};

    return crow::response{crow::NO_CONTENT};
}

} // namespace impl
} // namespace module
} // namespace flecs
