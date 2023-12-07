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

#include "daemon/modules/console/types/jwt.h"

namespace flecs {
namespace console {

auto jwt_t::token() const noexcept //
    -> const std::string&
{
    return _token;
}

auto jwt_t::token_expires() const noexcept //
    -> std::uint64_t
{
    return _token_expires;
}

auto from_json(const json_t& j, jwt_t& jwt) //
    -> void
{
    j.at("token").get_to(jwt._token);
    j.at("token_expires").get_to(jwt._token_expires);
}

auto to_json(json_t& j, const jwt_t& jwt) //
    -> void
{
    j = json_t({
        {"token", jwt.token()},
        {"token_expires", jwt.token_expires()},
    });
}

} // namespace console
} // namespace flecs
