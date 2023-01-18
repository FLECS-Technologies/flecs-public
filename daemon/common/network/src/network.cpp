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

#include "network.h"

#include <regex>

#include "util/cxx20/string.h"

namespace FLECS {

network_t::network_t()
    : network_t{""}
{}

network_t::network_t(std::string_view str)
    : _name{str}
    , _parent{}
    , _mac_address{}
    , _type{network_type_e::None}
{
    if (cxx20::contains(str, "-internal-")) {
        _type = network_type_e::Internal;
    } else if (cxx20::contains(str, "-ipvlan-")) {
        _type = network_type_e::IPVLAN;

        const auto adapter_regex = std::regex{"ipvlan-(.+)$"};
        auto m = std::cmatch{};
        std::regex_search(str.data(), m, adapter_regex);
        if ((m.size() > 1) && (m[1].matched)) {
            _parent = m[1];
        }
    } else if (cxx20::contains(str, "-macvlan-")) {
        _type = network_type_e::MACVLAN;

        const auto adapter_regex = std::regex{"macvlan-(.+)$"};
        auto m = std::cmatch{};
        std::regex_search(str.data(), m, adapter_regex);
        if ((m.size() > 1) && (m[1].matched)) {
            _parent = m[1];
        }
    } else if (!str.empty()) {
        _type = network_type_e::Bridge;
    }
}

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

auto network_t::mac_address() const // Copyright 2021-2023 FLECS Technologies GmbH
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
    noexcept //
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

} // namespace FLECS
