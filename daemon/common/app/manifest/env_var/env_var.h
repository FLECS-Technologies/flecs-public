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

#include <string>

#include "util/json/json.h"

namespace FLECS {

class env_var_t
{
public:
    env_var_t() = default;

    env_var_t(std::string var);

    auto is_valid() const noexcept //
        -> bool;

    auto var() const noexcept //
        -> const std::string&;

private:
    std::string _var;
};

class mapped_env_var_t
{
public:
    mapped_env_var_t() = default;

    mapped_env_var_t(env_var_t var, std::string value);

    mapped_env_var_t(std::string_view str);

    auto is_valid() const noexcept //
        -> bool;

    auto var() const noexcept //
        -> const std::string&;

    auto value() const noexcept //
        -> const std::string&;

private:
    friend auto to_json(json_t& json, const mapped_env_var_t& mapped_env_var) //
        -> void;

    friend auto from_json(const json_t& json, mapped_env_var_t& mapped_env_var) //
        -> void;

    env_var_t _env_var;
    std::string _value;
};

auto operator<(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator<(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator<=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator>(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator>=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator==(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;
auto operator!=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool;

auto to_string(const mapped_env_var_t& mapped_env_var) //
    -> std::string;

} // namespace FLECS
