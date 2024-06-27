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

#include "flecs/common/app/manifest/editor/editor.h"

namespace flecs {

editor_t::editor_t(std::string name, uint16_t port, bool supports_reverse_proxy)
    : _name(std::move(name))
    , _port(port)
    , _supports_reverse_proxy(supports_reverse_proxy)
{}

auto to_json(json_t& json, const editor_t& editor) //
    -> void
{
    json = json_t{
        {"name", editor._name},
        {"port", editor._port},
        {"supportsReverseProxy", editor._supports_reverse_proxy},
    };
}
auto from_json(const json_t& json, editor_t& editor) //
    -> void
{
    json.at("name").get_to(editor._name);
    json.at("port").get_to(editor._port);
    editor._supports_reverse_proxy = false;
    if (json.contains("supportsReverseProxy")) {
        json.at("supportsReverseProxy").get_to(editor._supports_reverse_proxy);
    }
}

} // namespace flecs