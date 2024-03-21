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

#include <cstdint>
#include <string>

#include "flecs/util/json/json.h"

namespace flecs {
namespace console {

class jwt_t
{
public:
    jwt_t();

    auto token() const noexcept //
        -> const std::string&;

    auto token_expires() const noexcept //
        -> std::uint64_t;

private:
    friend auto from_json(const json_t& j, jwt_t& jwt) //
        -> void;
    friend auto to_json(json_t& j, const jwt_t& jwt) //
        -> void;

    std::string _token;
    std::uint64_t _token_expires;
};

} // namespace console
} // namespace flecs
