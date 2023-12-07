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

#include "util/json/json.h"

namespace flecs {

class conffile_t
{
public:
    conffile_t()
        : _local{}
        , _container{}
        , _ro{}
        , _init{}
    {}

    explicit conffile_t(std::string_view str);

    auto local() const noexcept //
        -> const std::string&;
    auto local(std::string local) //
        -> void;

    auto container() const noexcept //
        -> const std::string&;
    auto container(std::string container) //
        -> void;

    auto ro() const noexcept //
        -> bool;
    auto ro(bool ro) //
        -> void;

    auto init() const noexcept //
        -> bool;
    auto init(bool init) //
        -> void;

    auto is_valid() const noexcept //
        -> bool;

private:
    friend auto to_json(json_t& json, const conffile_t& conffile) //
        -> void;
    friend auto from_json(const json_t& json, conffile_t& conffile) //
        -> void;

    std::string _local;
    std::string _container;
    bool _ro;
    bool _init;
};

auto to_string(const conffile_t& conffile) //
    -> std::string;

auto operator<(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator<(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator<=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator>(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator>=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator==(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;
auto operator!=(const conffile_t& lhs, const conffile_t& rhs) //
    -> bool;

} // namespace flecs
