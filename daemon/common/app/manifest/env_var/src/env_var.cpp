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

#include "env_var.h"

#include <regex>

#include "util/string/string_utils.h"

namespace FLECS {

env_var_t::env_var_t(std::string var)
    : _var{std::move(var)}
{}

auto env_var_t::is_valid() const noexcept //
    -> bool
{
    const auto name_regex = std::regex{"[a-zA-Z]+[a-zA-Z0-9_]*"};

    if (std::regex_match(_var, name_regex)) {
        return true;
    }

    return false;
}

auto env_var_t::var() const noexcept //
    -> const std::string&
{
    return _var;
}

mapped_env_var_t::mapped_env_var_t(env_var_t var, std::string value)
    : _env_var{std::move(var)}
    , _value{std::move(value)}
{}

mapped_env_var_t::mapped_env_var_t(std::string_view str)
    : _env_var{}
    , _value{}
{
    auto parts = split(str, ':');
    if (parts.size() == 2) {
        _env_var = parts[0];
        _value = parts[1];
        return;
    }

    parts = split(str, '=');
    if (parts.size() == 2) {
        _env_var = parts[0];
        _value = parts[1];
        return;
    }
}

auto mapped_env_var_t::is_valid() const noexcept //
    -> bool
{
    return _env_var.is_valid();
}

auto mapped_env_var_t::var() const noexcept //
    -> const std::string&
{
    return _env_var.var();
}

auto mapped_env_var_t::value() const noexcept //
    -> const std::string&
{
    return _value;
}

auto to_json(json_t& j, const mapped_env_var_t& mapped_env_var) //
    -> void
{
    j = json_t(to_string(mapped_env_var));
}

auto from_json(const json_t& j, mapped_env_var_t& mapped_env_var) //
    -> void
{
    try {
        mapped_env_var = mapped_env_var_t{j.get<std::string_view>()};
    } catch (...) {
        mapped_env_var = mapped_env_var_t{};
    }
}

auto operator<(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return lhs.var() < rhs.var();
}

auto operator<=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return lhs.var() == rhs.var();
}

auto operator!=(const mapped_env_var_t& lhs, const mapped_env_var_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

auto to_string(const mapped_env_var_t& mapped_env_var) //
    -> std::string
{
    return mapped_env_var.is_valid()
               ? stringify_delim('=', mapped_env_var.var(), mapped_env_var.value())
               : std::string{};
}

} // namespace FLECS
