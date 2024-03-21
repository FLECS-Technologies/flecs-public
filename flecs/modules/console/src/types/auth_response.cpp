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

#include "flecs/modules/console/types/auth_response.h"

namespace flecs {
namespace console {

auto auth_response_data_t::user() const noexcept //
    -> const user_t&
{
    return _user;
}

auto auth_response_data_t::jwt() const noexcept //
    -> const jwt_t&
{
    return _jwt;
}

auto auth_response_data_t::feature_flags() const noexcept //
    -> const feature_flags_t&
{
    return _ff;
}

auto from_json(const json_t& j, auth_response_data_t& response) //
    -> void
{
    j.at("user").get_to(response._user);
    j.at("jwt").get_to(response._jwt);
    j.at("feature_flags").get_to(response._ff);
}

auto to_json(json_t& j, const auth_response_data_t& response) //
    -> void
{
    j = json_t(
        {{"user", response.user()}, {"jwt", response.jwt()}, {"feature_flags", response.feature_flags()}});
}

auto from_json(const json_t& j, auth_response_t& auth_response) //
    -> void
{
    from_json(j, static_cast<base_response_t&>(auth_response));
    from_json(j.at("data"), static_cast<auth_response_data_t&>(auth_response));
}

auto to_json(json_t& j, const auth_response_t& auth_response) //
    -> void
{
    to_json(j, static_cast<const base_response_t&>(auth_response));
    to_json(j["data"], static_cast<const auth_response_data_t&>(auth_response));
}

} // namespace console
} // namespace flecs
