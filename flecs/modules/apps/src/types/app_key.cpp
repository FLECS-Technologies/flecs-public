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

#include "flecs/modules/apps/types/app_key.h"

namespace flecs {
namespace apps {

key_t::key_t(std::tuple<name_t, std::string> app_key)
    : _key{std::move(app_key)}
{}

key_t::key_t(std::string app_name, std::string app_version)
    : key_t{std::make_tuple(name_t{std::move(app_name)}, std::move(app_version))}
{}

key_t::key_t(name_t app_name, std::string app_version)
    : key_t{std::make_tuple(std::move(app_name), std::move(app_version))}
{}

auto key_t::is_valid() const noexcept //
    -> bool
{
    return std::get<0>(_key).is_valid() && !std::get<1>(_key).empty();
}

auto key_t::name() const noexcept //
    -> const std::string&
{
    return std::get<0>(_key).value();
}

auto key_t::version() const noexcept //
    -> const std::string&
{
    return std::get<1>(_key);
}

void to_json(json_t& j, const key_t& app_key)
{
    j = json_t({
        {"name", app_key.name()},
        {"version", app_key.version()},
    });
}

void from_json(const json_t& json, key_t& app_key)
{
    app_key = key_t{json.at("name"), json.at("version")};
}

auto to_string(const key_t& app_key) //
    -> std::string
{
    using std::operator""s;
    return app_key.name().data() + " ("s + app_key.version().data() + ")"s;
}

} // namespace apps
} // namespace flecs
