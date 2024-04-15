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

#include "flecs/common/app/manifest/variable/variable.h"

#include <regex>

#include "flecs/util/string/string_utils.h"

namespace flecs {

var_t::var_t(std::string var)
    : _var{std::move(var)}
{}

auto var_t::is_valid() const noexcept //
    -> bool
{
    return !_var.empty();
}

auto var_t::var() const noexcept //
    -> const std::string&
{
    return _var;
}

auto var_t::parse_env_var_name(std::string str) //
    -> std::optional<var_t>
{
    static const auto name_regex = std::regex{R"(^[a-zA-Z]+(?:[a-zA-Z0-9_\-\.])*$)"};

    if (std::regex_match(str, name_regex)) {
        return var_t{std::move(str)};
    }
    return {};
}

auto var_t::parse_label_var_name(std::string str) //
    -> std::optional<var_t>
{
    // consists of letters, digits, '-' and '.'
    // starts and ends with lowercase letter
    // consecutive '.' and '-' are not allowed
    static const auto name_regex = std::regex{R"(^[a-z](?:(?:[\-\.]?[a-zA-Z0-9])*[\-\.]?[a-z])?$)"};

    if (std::regex_match(str, name_regex)) {
        return var_t{std::move(str)};
    }
    return {};
}

mapped_var_t::mapped_var_t(var_t var, std::string value)
    : _env_var{std::move(var)}
    , _value{std::move(value)}
{}

auto mapped_var_t::is_valid() const noexcept //
    -> bool
{
    return _env_var.is_valid();
}

auto mapped_var_t::var() const noexcept //
    -> const std::string&
{
    return _env_var.var();
}

auto mapped_var_t::value() const noexcept //
    -> const std::string&
{
    return _value;
}

auto to_json(json_t& j, const mapped_var_t& mapped_env_var) //
    -> void
{
    j = json_t(to_string(mapped_env_var));
}

auto mapped_env_var_t::try_parse(std::string_view str) //
    -> std::optional<mapped_env_var_t>
{
    auto [var, val] = split_first(str, ':');
    auto var_name = var_t::parse_env_var_name(std::move(var));
    if (var_name.has_value()) {
        return mapped_env_var_t{var_name.value(), val};
    }

    std::tie(var, val) = split_first(str, '=');
    var_name = var_t::parse_env_var_name(std::move(var));
    if (var_name.has_value()) {
        return mapped_env_var_t{var_name.value(), val};
    }
    return {};
}

auto mapped_label_var_t::try_parse(std::string_view str) //
    -> std::optional<mapped_label_var_t>
{
    auto [var, val] = split_first(str, '=');
    auto var_name = var_t::parse_label_var_name(std::move(var));
    if (var_name.has_value()) {
        return mapped_label_var_t{var_name.value(), val};
    }
    return {};
}

auto mapped_var_t::operator<=>(const mapped_var_t& other) const //
    -> std::strong_ordering
{
    return var() <=> other.var();
}

auto mapped_var_t::operator==(const mapped_var_t& other) const //
    -> bool
{
    return (*this) <=> other == std::strong_ordering::equal;
}

auto mapped_var_t::operator!=(const mapped_var_t& other) const //
    -> bool
{
    return !((*this) == other);
}

auto to_string(const mapped_var_t& mapped_env_var) //
    -> std::string
{
    return mapped_env_var.is_valid() ? stringify_delim('=', mapped_env_var.var(), mapped_env_var.value())
                                     : std::string{};
}

auto from_json(const json_t& j, mapped_env_var_t& mapped_env_var) //
    -> void
{
    try {
        auto parse_result = mapped_env_var_t::try_parse(j.get<std::string_view>());
        if (parse_result.has_value()) {
            mapped_env_var = parse_result.value_or(mapped_env_var_t{});
            return;
        }
    } catch (...) {
        mapped_env_var = mapped_env_var_t{};
    }
}

mapped_env_var_t::mapped_env_var_t(var_t var, std::string value)
    : mapped_var_t(std::move(var), std::move(value))
{}

auto from_json(const json_t& j, mapped_label_var_t& mapped_env_var) //
    -> void
{
    auto str = j.get<std::string_view>();
    auto parse_result = mapped_label_var_t::try_parse(str);
    if (parse_result.has_value()) {
        mapped_env_var = parse_result.value();
        return;
    }
    throw std::invalid_argument{"Invalid value for label: " + std::string{str}};
}

mapped_label_var_t::mapped_label_var_t(var_t var, std::string value)
    : mapped_var_t(std::move(var), std::move(value))
{}

} // namespace flecs
