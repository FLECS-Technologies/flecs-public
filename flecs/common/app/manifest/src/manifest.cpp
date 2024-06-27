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
    , _labels{}
{}

app_manifest_t app_manifest_t::from_json(const json_t& json)
{
    auto res = app_manifest_t{};

    try {
        json.get_to(res);
    } catch (...) {
    }

    res.upgrade_manifest_version();
    return res;
}
app_manifest_t app_manifest_t::from_yaml(const yaml_t& yaml)
{
    auto res = app_manifest_t{};
    res.parse_yaml(yaml);
    res.upgrade_manifest_version();
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
        auto error_found = false;
        REQUIRED_TYPED_YAML_VALUE(yaml, app, _app);
        OPTIONAL_TYPED_YAML_VALUE(yaml, _schemaVersion, _manifest_version);
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
            auto parse_result = mapped_env_var_t::try_parse(env.as<std::string>());
            if (parse_result.has_value()) {
                _env.emplace(parse_result.value());
            } else {
                error_found = true;
            }
        }

        OPTIONAL_YAML_NODE(yaml, labels, labels);
        for (const auto& label : labels) {
            auto parse_result = mapped_label_var_t::try_parse(label.as<std::string>());
            if (parse_result.has_value()) {
                _labels.emplace(parse_result.value());
            } else {
                error_found = true;
            }
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
        if (error_found) {
            _valid = false;
        } else {
            validate();
        }
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

void app_manifest_t::upgrade_manifest_version()
{
    static const char* LATEST_VERSION = "2.1.0";
    auto current_version = _manifest_version.empty() ? "2.0.0" : _manifest_version;
    if (current_version == LATEST_VERSION) {
        return;
    }

    // Current version is 2.1
    if (current_version.starts_with("2.0.")) {
        // Meaning of "editor" changed from 2.0 to 2.1, "editor" now specifies port at container not at host
        // therefore mapping from host to container via "ports" is not necessary
        if (!_editor.empty()) {
            auto port = uint16_t{};
            try {
                port = std::stoi(_editor.substr(1));
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
            _editor = ":" + std::to_string(port);
        }
        _manifest_version = LATEST_VERSION;
    }
    std::cerr << "Upgraded manifest " << _app << " from " << current_version << " to " << _manifest_version << std::endl;
}


auto to_json(json_t& json, const app_manifest_t& app_manifest) //
    -> void
{
    json = json_t({
        {"app", app_manifest._app},
        {"_schemaVersion", app_manifest._manifest_version},
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

    OPTIONAL_JSON_VALUE(json, _schemaVersion, app_manifest._manifest_version);
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
