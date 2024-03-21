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

#include "base_response.h"
#include "feature_flags.h"
#include "flecs/util/json/json.h"
#include "jwt.h"
#include "user.h"

namespace flecs {
namespace console {

class auth_response_data_t
{
public:
    auto user() const noexcept //
        -> const user_t&;
    auto jwt() const noexcept //
        -> const jwt_t&;
    auto feature_flags() const noexcept //
        -> const feature_flags_t&;

private:
    friend auto from_json(const json_t& j, auth_response_data_t& ff) //
        -> void;
    friend auto to_json(json_t& j, const auth_response_data_t& ff) //
        -> void;

    user_t _user;
    jwt_t _jwt;
    feature_flags_t _ff;
};

class auth_response_t : public base_response_t, public auth_response_data_t
{
private:
    friend auto from_json(const json_t& j, auth_response_t& ff) //
        -> void;
    friend auto to_json(json_t& j, const auth_response_t& ff) //
        -> void;
};

} // namespace console
} // namespace flecs
