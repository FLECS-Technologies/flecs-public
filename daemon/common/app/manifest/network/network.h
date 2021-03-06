// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef DD9A8DC3_3CE8_4B4D_BCCD_6CE7E45FC419
#define DD9A8DC3_3CE8_4B4D_BCCD_6CE7E45FC419

#include <string_view>

#include "deployment/deployment.h"
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
        -> network_type_t;

    auto name(std::string name) //
        -> void;
    auto parent(std::string parent) //
        -> void;
    auto mac_address(std::string mac_address) //
        -> void;
    auto type(network_type_t type) noexcept //
        -> void;

    auto is_valid() const noexcept //
        -> bool;

private:
    friend auto to_json(json_t& j, const network_t& conffile) //
        -> void;

    std::string _name;
    std::string _parent;
    std::string _mac_address;
    network_type_t _type;
};

} // namespace FLECS

#endif // DD9A8DC3_3CE8_4B4D_BCCD_6CE7E45FC419
