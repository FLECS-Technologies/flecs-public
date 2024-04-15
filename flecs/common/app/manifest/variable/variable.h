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

#include "flecs/util/json/json.h"

namespace flecs {

class var_t
{
public:
    var_t() = default;

    auto is_valid() const noexcept //
        -> bool;

    auto var() const noexcept //
        -> const std::string&;

    static auto parse_env_var_name(std::string str) //
        -> std::optional<var_t>;

    static auto parse_label_var_name(std::string str) //
        -> std::optional<var_t>;

private:
    var_t(std::string var);

    std::string _var;
};

class mapped_var_t
{
public:
    mapped_var_t() = default;

    auto is_valid() const noexcept //
        -> bool;

    auto var() const noexcept //
        -> const std::string&;

    auto value() const noexcept //
        -> const std::string&;

    auto operator<=>(const mapped_var_t& other) const //
        -> std::strong_ordering;
    auto operator==(const mapped_var_t& other) const //
        -> bool;
    auto operator!=(const mapped_var_t& other) const //
        -> bool;

    mapped_var_t(var_t var, std::string value);
private:

    friend auto to_json(json_t& json, const mapped_var_t& mapped_env_var) //
        -> void;

    var_t _env_var;
    std::string _value;
};


auto to_string(const mapped_var_t& mapped_env_var) //
    -> std::string;

class mapped_env_var_t : public mapped_var_t
{
public:
    mapped_env_var_t() = default;
    mapped_env_var_t(var_t var, std::string value);
    static auto try_parse(std::string_view str) //
        -> std::optional<mapped_env_var_t>;
private:
    friend auto from_json(const json_t& json, mapped_env_var_t& mapped_env_var) //
        -> void;
};

class mapped_label_var_t : public mapped_var_t
{
public:
    mapped_label_var_t() = default;
    mapped_label_var_t(var_t var, std::string value);
    static auto try_parse(std::string_view str) //
        -> std::optional<mapped_label_var_t>;
private:
    friend auto from_json(const json_t& json, mapped_label_var_t& mapped_env_var) //
        -> void;
};
} // namespace flecs
