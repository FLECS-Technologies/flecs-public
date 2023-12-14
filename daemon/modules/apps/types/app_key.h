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
#include <string_view>
#include <tuple>

#include "app_name.h"
#include "util/json/json.h"

namespace flecs {
namespace apps {

class key_t
{
public:
    key_t() = default;
    key_t(std::tuple<name_t, std::string> app_key);
    key_t(std::string app_name, std::string app_version);
    key_t(name_t app_name, std::string app_version);

    auto is_valid() const noexcept //
        -> bool;

    auto name() const noexcept //
        -> std::string_view;

    auto version() const noexcept //
        -> std::string_view;

private:
    friend auto operator<=>(const key_t&, const key_t&) = default;

    friend auto to_json(json_t& j, const key_t& app_key) //
        -> void;
    friend auto from_json(const json_t& json, key_t& app_key) //
        -> void;

    std::tuple<name_t, std::string> _key;
};

auto to_string(const key_t& app_key) //
    -> std::string;

} // namespace apps
} // namespace flecs
