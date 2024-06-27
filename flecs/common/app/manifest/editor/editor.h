// Copyright 2021-2024 FLECS Technologies GmbH
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

#include "flecs/util/json/json.h"

namespace flecs {

class editor_t
{
public:

    editor_t() = default;

    editor_t(std::string name, uint16_t port, bool supports_reverse_proxy);

    auto& name() const noexcept { return _name; }
    auto port() const noexcept { return _port; }
    auto supports_reverse_proxy() const noexcept { return _supports_reverse_proxy; }
private:
    friend auto to_json(json_t& json, const editor_t& app_manifest) //
        -> void;
    friend auto from_json(const json_t& json, editor_t& app_manifest) //
        -> void;
    std::string _name;
    uint16_t _port;
    bool _supports_reverse_proxy;
};

} // namespace flecs

