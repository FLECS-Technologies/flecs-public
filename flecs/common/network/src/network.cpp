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

#include "flecs/common/network/network.h"

#include <regex>

#include "flecs/util/cxx23/string.h"

namespace flecs {

network_t::network_t()
    : _name{}
    , _parent{}
    , _mac_address{}
    , _type{network_type_e::None}
{}

auto network_t::name() const noexcept //
    -> const std::string&
{
    return _name;
}

auto network_t::parent() const noexcept //
    -> const std::string&
{
    return _parent;
}

auto network_t::mac_address() const noexcept //
    -> const std::string&
{
    return _mac_address;
}

auto network_t::type() const noexcept //
    -> network_type_e
{
    return _type;
}

auto network_t::name(std::string name) //
    -> void
{
    _name = name;
}

auto network_t::parent(std::string parent) //
    -> void
{
    _parent = parent;
}

auto network_t::mac_address(std::string mac_address) //
    -> void
{
    _mac_address = mac_address;
}

auto network_t::type(network_type_e type) noexcept //
    -> void
{
    _type = type;
}

auto network_t::is_valid() const noexcept //
    -> bool
{
    return !(_type == network_type_e::None);
}

auto to_json(json_t& json, const network_t& network) //
    -> void
{
    json = json_t{
        {"mac_address", network.mac_address()},
        {"name", network.name()},
        {"parent", network.parent()},
        {"type", to_string(network.type())},
    };
}

auto from_json(const json_t& json, network_t& network) //
    -> void
{
    json.at("mac_address").get_to(network._mac_address);
    json.at("name").get_to(network._name);
    json.at("parent").get_to(network._parent);
    auto type = std::string{};
    json.at("type").get_to(type);
    network._type = network_type_from_string(type);
}

} // namespace flecs
