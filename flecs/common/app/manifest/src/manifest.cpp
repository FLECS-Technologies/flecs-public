// Copyright 2021-2023 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License")
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

#include "flecs/common/app/manifest/manifest.h"

#include <fstream>
#include <iostream>

namespace flecs {

#define REQUIRED_JSON_VALUE(json, key, target) \
    do {                                       \
        json.at(#key).get_to(target);          \
    } while (false)

#define OPTIONAL_JSON_VALUE(json, key, target) \
    if (json.contains(#key)) {                 \
        json.at(#key).get_to(target);          \
    }

app_manifest_t::app_manifest_t()
    : _valid{}
    , _app{}
    , _args{}
    , _conffiles{}
    , _devices{}
    , _editors{}
    , _env{}
    , _hostname{}
    , _image{}
    , _interactive{}
    , _multi_instance{}
    , _networks{}
    , _ports{}
    , _version{}
    , _volumes{}
    , _labels{}
{}

app_manifest_t app_manifest_t::from_json(const json_t& json)
{
    auto res = app_manifest_t{};

    try {
        json.get_to(res);
    } catch (...) {
    }

    return res;
}

app_manifest_t app_manifest_t::from_json_string(std::string_view string)
{
    return from_json(parse_json(string));
}

app_manifest_t app_manifest_t::from_json_file(const fs::path& path)
{
    auto res = app_manifest_t{};

    auto json_file = std::ifstream{path};
    if (!json_file) {
        return {};
    }
    return from_json(parse_json(json_file));
}

void app_manifest_t::validate()
{
    _valid = false;
    for (const auto& conffile : _conffiles) {
        if (!conffile.is_valid()) {
            return;
        }
    }

    for (const auto& env : _env) {
        if (!env.is_valid()) {
            return;
        }
    }

    for (const auto& port : _ports) {
        if (!port.is_valid()) {
            return;
        }
    }

    for (const auto& volume : _volumes) {
        if (!volume.is_valid()) {
            return;
        }
    }

    if (!_hostname.empty() && _multi_instance) {
        return;
    }

    _valid = true;
}

auto to_json(json_t& json, const app_manifest_t& app_manifest) //
    -> void
{
    auto editors = json_t::array_t {};
    for (const auto& [_, editor] : app_manifest.editors()) {
        editors.emplace_back(editor);
    }
    json = json_t({
        {"app", app_manifest._app},
        {"_schemaVersion", app_manifest._manifest_version},
        {"version", app_manifest._version},
        {"image", app_manifest._image},

        {"multiInstance", app_manifest._multi_instance},
        {"editors", editors},

        {"args", app_manifest._args},
        {"capabilities", app_manifest._capabilities},
        {"conffiles", app_manifest._conffiles},
        {"devices", app_manifest._devices},
        {"env", app_manifest._env},
        {"hostname", app_manifest._hostname},
        {"interactive", app_manifest._interactive},
        {"networks", app_manifest._networks},
        {"ports", app_manifest._ports},
        {"startupOptions", app_manifest._startup_options},
        {"volumes", app_manifest._volumes},
        {"labels", app_manifest._labels},
    });
}

auto from_json(const json_t& json, app_manifest_t& app_manifest) //
    -> void
{
    static const char* LATEST_VERSION = "3.0.0";
    auto current_version = std::string{};
    OPTIONAL_JSON_VALUE(json, _schemaVersion, current_version);
    current_version = current_version.empty() ? "2.0.0" : current_version;

    if (current_version.starts_with("2.")) {
        from_json_2(json, app_manifest);
        app_manifest._manifest_version = LATEST_VERSION;
    } else if (current_version.starts_with("3.")) {
        from_json_3(json, app_manifest);
        app_manifest._manifest_version = LATEST_VERSION;
    }
}

auto from_json_common(const json_t& json, app_manifest_t& app_manifest) //
    -> void
{
    REQUIRED_JSON_VALUE(json, app, app_manifest._app);
    REQUIRED_JSON_VALUE(json, version, app_manifest._version);
    REQUIRED_JSON_VALUE(json, image, app_manifest._image);
    OPTIONAL_JSON_VALUE(json, multiInstance, app_manifest._multi_instance);

    OPTIONAL_JSON_VALUE(json, args, app_manifest._args);
    OPTIONAL_JSON_VALUE(json, capabilities, app_manifest._capabilities);
    OPTIONAL_JSON_VALUE(json, conffiles, app_manifest._conffiles);
    OPTIONAL_JSON_VALUE(json, devices, app_manifest._devices);
    OPTIONAL_JSON_VALUE(json, env, app_manifest._env);
    OPTIONAL_JSON_VALUE(json, hostname, app_manifest._hostname);
    OPTIONAL_JSON_VALUE(json, interactive, app_manifest._interactive);
    OPTIONAL_JSON_VALUE(json, networks, app_manifest._networks);
    OPTIONAL_JSON_VALUE(json, ports, app_manifest._ports);
    OPTIONAL_JSON_VALUE(json, startupOptions, app_manifest._startup_options);
    OPTIONAL_JSON_VALUE(json, volumes, app_manifest._volumes);
    OPTIONAL_JSON_VALUE(json, labels, app_manifest._labels);
}

auto from_json_2(const json_t& json, app_manifest_t& app_manifest) //
    -> void
{
    auto editor = std::string{};
    OPTIONAL_JSON_VALUE(json, editor, editor);
    from_json_common(json, app_manifest);
    app_manifest.editor_to_editors(editor);
    app_manifest.validate();
}

auto from_json_3(const json_t& json, app_manifest_t& app_manifest) //
    -> void
{
    auto editors = std::vector<editor_t>{};
    app_manifest._editors.clear();
    OPTIONAL_JSON_VALUE(json, editors, editors);
    for (editor_t editor : editors) {
        app_manifest._editors.insert({editor.port(), std::move(editor)});
    }
    from_json_common(json, app_manifest);
    app_manifest.validate();
}

auto app_manifest_t::editor_to_editors(const std::string& editor) //
    -> void
{
    _editors = {};
    // Meaning of "editor" changed, "editor" now specifies port at container not at host
    // therefore mapping from host to container via "ports" is not necessary
    if (!editor.empty()) {
        auto port = uint16_t{};
        try {
            port = std::stoi(editor.substr(1));
        } catch (std::exception const& e) {
            return;
        }
        // Check if the published ports of the instance contain the editor port
        for (auto port_mapping = _ports.begin(); port_mapping != _ports.end(); port_mapping++) {
            if (port_mapping->host_port_range().start_port() == port) {
                port = port_mapping->container_port_range().start_port();
                _ports.erase(port_mapping);
                break;
            }
        }
        _editors = {{port, editor_t{"", port, false}}};
    }
}

} // namespace flecs
