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
    , _editor{}
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
    json = json_t({
        {"app", app_manifest._app},
        {"version", app_manifest._version},
        {"image", app_manifest._image},

        {"multiInstance", app_manifest._multi_instance},
        {"editor", app_manifest._editor},

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
    REQUIRED_JSON_VALUE(json, app, app_manifest._app);
    REQUIRED_JSON_VALUE(json, version, app_manifest._version);
    REQUIRED_JSON_VALUE(json, image, app_manifest._image);

    OPTIONAL_JSON_VALUE(json, multiInstance, app_manifest._multi_instance);
    OPTIONAL_JSON_VALUE(json, editor, app_manifest._editor);

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

    app_manifest.validate();
}

} // namespace flecs
