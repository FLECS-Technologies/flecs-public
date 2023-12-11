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

#include "daemon/modules/console/types/user.h"

namespace flecs {
namespace console {

user_t::user_t()
    : _id{}
    , _user_email{}
    , _user_login{}
    , _display_name{}
{}

auto user_t::id() const noexcept //
    -> std::uint64_t
{
    return _id;
}

auto user_t::user_email() const noexcept //
    -> const std::string&
{
    return _user_email;
}

auto user_t::user_login() const noexcept //
    -> const std::string&
{
    return _user_login;
}

auto user_t::display_name() const noexcept //
    -> const std::string&
{
    return _display_name;
}

auto from_json(const json_t& j, user_t& user) //
    -> void
{
    j.at("ID").get_to(user._id);
    j.at("user_email").get_to(user._user_email);
    j.at("user_login").get_to(user._user_login);
    j.at("display_name").get_to(user._display_name);
}

auto to_json(json_t& j, const user_t& user) //
    -> void
{
    j = json_t({
        {"ID", user.id()},
        {"user_email", user.user_email()},
        {"user_login", user.user_login()},
        {"display_name", user.display_name()},
    });
}

} // namespace console
} // namespace flecs
