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

#include "daemon/modules/console/types/access_token.h"

namespace flecs {
namespace console {

auto access_token_t::username() const noexcept //
    -> const std::string&
{
    return _username;
}

auto access_token_t::password() const noexcept //
    -> const std::string&
{
    return _password;
}

auto from_json(const json_t& j, access_token_t& access_token) //
    -> void
{
    j.at("username").get_to(access_token._username);
    j.at("password").get_to(access_token._password);
}

auto to_json(json_t& j, const access_token_t& access_token) //
    -> void
{
    j = json_t({
        {"username", access_token.username()},
        {"password", access_token.password()},
    });
}

} // namespace console
} // namespace flecs
