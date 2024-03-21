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

#define REQUIRED_TYPED_YAML_VALUE(yaml, value, target) \
    do {                                               \
        target = yaml[#value].as<decltype(target)>();  \
    } while (false)

#define REQUIRED_YAML_VALUE(yaml, value, target) \
    do {                                         \
        target = yaml[#value];                   \
    } while (false)

#define OPTIONAL_TYPED_YAML_VALUE(yaml, value, target)    \
    do {                                                  \
        try {                                             \
            target = yaml[#value].as<decltype(target)>(); \
        } catch (const YAML::Exception& ex) {             \
        }                                                 \
    } while (false)

#define OPTIONAL_YAML_NODE(yaml, value, target) \
    auto target = yaml_t{};                     \
    do {                                        \
        try {                                   \
            target = yaml[#value];              \
        } catch (const YAML::Exception& ex) {   \
        }                                       \
    } while (false)

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
app_manifest_t app_manifest_t::from_yaml(const yaml_t& yaml)
{
    auto res = app_manifest_t{};
    res.parse_yaml(yaml);
    return res;
}

app_manifest_t app_manifest_t::from_json_string(std::string_view string)
{
    return from_json(parse_json(string));
}
app_manifest_t app_manifest_t::from_yaml_string(std::string_view string)
{
    try {
        return from_yaml(yaml_from_string(std::move(string)));
    } catch (const std::exception& ex) {
        std::fprintf(stderr, "Could not open manifest: %s\n", ex.what());
    }
    return {};
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
app_manifest_t app_manifest_t::from_yaml_file(const fs::path& path)
{
    try {
        return from_yaml(yaml_from_file(path.c_str()));
    } catch (const std::exception& ex) {
        std::fprintf(stderr, "Could not open manifest %s: %s\n", path.c_str(), ex.what());
    }
    return {};
}

void app_manifest_t::parse_yaml(const yaml_t& yaml)
{
    try {
        REQUIRED_TYPED_YAML_VALUE(yaml, app, _app);
        OPTIONAL_TYPED_YAML_VALUE(yaml, args, _args);

        OPTIONAL_YAML_NODE(yaml, capabilities, capabilities);
        for (const auto& cap : capabilities) {
            _capabilities.push_back(cap.as<std::string>());
        }
        OPTIONAL_YAML_NODE(yaml, conffiles, conffiles);
        for (const auto& conf : conffiles) {
            _conffiles.emplace_back(conffile_t{conf.as<std::string>()});
        }

        OPTIONAL_YAML_NODE(yaml, devices, devices);
        for (const auto& device : devices) {
            _devices.emplace(device.as<std::string>());
        }
        OPTIONAL_TYPED_YAML_VALUE(yaml, editor, _editor);

        OPTIONAL_YAML_NODE(yaml, env, envs);
        for (const auto& env : envs) {
            _env.emplace(mapped_env_var_t{env.as<std::string>()});
        }

        OPTIONAL_TYPED_YAML_VALUE(yaml, hostname, _hostname);
        REQUIRED_TYPED_YAML_VALUE(yaml, image, _image);
        OPTIONAL_TYPED_YAML_VALUE(yaml, interactive, _interactive);
        OPTIONAL_TYPED_YAML_VALUE(yaml, multiInstance, _multi_instance);

        _networks.emplace_back("flecs");

        OPTIONAL_YAML_NODE(yaml, networkSettings, network_settings);
        for (const auto& setting : network_settings) {
            auto mac_address = std::string{};
            OPTIONAL_TYPED_YAML_VALUE(setting, macAddress, mac_address);
            if (!mac_address.empty()) {
                _networks.rbegin()->mac_address(mac_address);
            }
        }

        OPTIONAL_YAML_NODE(yaml, ports, ports);
        for (const auto& port_range : ports) {
            _ports.emplace_back(mapped_port_range_t{port_range.as<std::string>().data()});
        }

        OPTIONAL_YAML_NODE(yaml, startupOptions, startup_options);
        for (const auto& startup_option : startup_options) {
            _startup_options.emplace_back(startup_option_from_string(startup_option.as<std::string>()));
        }

        REQUIRED_TYPED_YAML_VALUE(yaml, version, _version);

        OPTIONAL_YAML_NODE(yaml, volumes, volumes);
        for (const auto& volume : volumes) {
            _volumes.emplace_back(volume_t{volume.as<std::string>()});
        }

        validate();
    } catch (const std::exception& ex) {
        std::fprintf(stderr, "Could not open manifest: Invalid YAML (%s)\n", ex.what());
        *this = app_manifest_t{};
    }
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
    json = json_t(
        {{"app", app_manifest._app},
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
         {"volumes", app_manifest._volumes}});
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

    app_manifest.validate();
}

} // namespace flecs
