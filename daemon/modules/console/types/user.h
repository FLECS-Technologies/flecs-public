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

#include <cinttypes>
#include <string>

#include "util/json/json.h"

namespace flecs {
namespace console {

class user_t
{
public:
    auto id() const noexcept //
        -> std::uint64_t;
    auto user_email() const noexcept //
        -> const std::string&;
    auto user_login() const noexcept //
        -> const std::string&;
    auto display_name() const noexcept //
        -> const std::string&;

private:
    friend auto from_json(const json_t& j, user_t& user) //
        -> void;
    friend auto to_json(json_t& j, const user_t& user) //
        -> void;

    std::uint64_t _id;
    std::string _user_email;
    std::string _user_login;
    std::string _display_name;
};

} // namespace console
} // namespace flecs
