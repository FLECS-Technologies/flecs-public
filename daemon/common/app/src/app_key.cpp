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

#include "app_key.h"

namespace FLECS {

app_key_t::app_key_t(std::tuple<app_name_t, std::string> app_key)
    : _key{std::move(app_key)}
{}

app_key_t::app_key_t(std::string app_name, std::string app_version)
    : app_key_t{std::make_tuple(app_name_t{std::move(app_name)}, std::move(app_version))}
{}

app_key_t::app_key_t(app_name_t app_name, std::string app_version)
    : app_key_t{std::make_tuple(std::move(app_name), std::move(app_version))}
{}

auto app_key_t::is_valid() const noexcept //
    -> bool
{
    return std::get<0>(_key).is_valid() && !std::get<1>(_key).empty();
}

auto app_key_t::name() const noexcept //
    -> std::string_view
{
    return std::get<0>(_key).value();
}

auto app_key_t::version() const noexcept //
    -> std::string_view
{
    return std::get<1>(_key);
}

auto operator<(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return lhs._key < rhs._key;
}

auto operator<=(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return !(lhs > rhs);
}

auto operator>(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return rhs < lhs;
}

auto operator>=(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return !(lhs < rhs);
}

auto operator==(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return lhs._key == rhs._key;
}

auto operator!=(const app_key_t& lhs, const app_key_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
}

void to_json(json_t& j, const app_key_t& app_key)
{
    j = json_t({{"name", app_key.name()}, {"version", app_key.version()}});
}

void from_json(const json_t& json, app_key_t& app_key)
{
    app_key = app_key_t{json.at("name"), json.at("version")};
}

} // namespace FLECS
