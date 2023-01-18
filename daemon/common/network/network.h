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

#include <string_view>

#include "network_type.h"
#include "util/json/json.h"

namespace FLECS {

class network_t
{
public:
    network_t();

    explicit network_t(std::string_view str);

    auto name() const noexcept //
        -> const std::string&;
    auto parent() const noexcept //
        -> const std::string&;
    auto mac_address() const noexcept //
        -> const std::string&;
    auto type() const noexcept //
        -> network_type_e;

    auto name(std::string name) //
        -> void;
    auto parent(std::string parent) //
        -> void;
    auto mac_address(std::string mac_address) //
        -> void;
    auto type(network_type_e type) noexcept //
        -> void;

    auto is_valid() const noexcept //
        -> bool;

private:
    friend auto to_json(json_t& json, const network_t& network) //
        -> void;
    friend auto from_json(const json_t& json, network_t& network) //
        -> void;

    std::string _name;
    std::string _parent;
    std::string _mac_address;
    network_type_e _type;
};

} // namespace FLECS
