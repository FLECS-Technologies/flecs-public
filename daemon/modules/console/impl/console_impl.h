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

#include "daemon/modules/console/console.h"

namespace flecs {
namespace module {
namespace impl {

class console_t
{
    friend class flecs::module::console_t;

public:
    console_t();

    ~console_t();

private:
    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    auto do_authentication() const noexcept //
        -> const console::auth_response_t&;

    auto do_activate_license(std::string_view session_id) //
        -> result_t;

    auto do_validate_license(std::string_view session_id) //
        -> result_t;

    auto do_store_authentication(console::auth_response_t auth) //
        -> crow::response;
    auto do_delete_authentication() //
        -> crow::response;

    console::auth_response_t _auth;
};

} // namespace impl
} // namespace module
} // namespace flecs
