// Copyright 2021-2022 FLECS Technologies GmbH
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

#include "manifest.h"

#include "util/cxx20/string.h"

namespace FLECS {

#define REQUIRED_TYPED_YAML_VALUE(yaml, value, target) \
    do                                                 \
    {                                                  \
        target = yaml[#value].as<decltype(target)>();  \
    } while (false)

#define REQUIRED_YAML_VALUE(yaml, value, target) \
    do                                           \
    {                                            \
        target = yaml[#value];                   \
    } while (false)

#define OPTIONAL_TYPED_YAML_VALUE(yaml, value, target)    \
    do                                                    \
    {                                                     \
        try                                               \
        {                                                 \
            target = yaml[#value].as<decltype(target)>(); \
        }                                                 \
        catch (const YAML::Exception& ex)                 \
        {}                                                \
    } while (false)

#define OPTIONAL_YAML_NODE(yaml, value, target) \
    auto target = yaml_t{};                     \
    do                                          \
    {                                           \
        try                                     \
        {                                       \
            target = yaml[#value];              \
        }                                       \
        catch (const YAML::Exception& ex)       \
        {}                                      \
    } while (false)

app_manifest_t::app_manifest_t()
    : _yaml_loaded{}
    , _yaml_valid{}
    , _app{}
    , _args{}
    , _author{}
    , _avatar{}
    , _category{}
    , _conffiles{}
    , _description{}
    , _devices{}
    , _editor{}
    , _env{}
    , _hostname{}
    , _image{}
    , _interactive{}
    , _multi_instance{}
    , _networks{}
    , _ports{}
    , _title{}
    , _version{}
    , _volumes{}
{}

app_manifest_t app_manifest_t::from_yaml_file(const fs::path& path)
{
    auto res = app_manifest_t{};
    try
    {
        const auto yaml = yaml_from_file(path);
        res.parse_yaml(yaml);
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest %s: Invalid YAML (%s)\n", path.c_str(), ex.what());
    }
    return res;
}

app_manifest_t app_manifest_t::from_yaml_string(const std::string& str)
{
    auto res = app_manifest_t{};
    try
    {
        const auto yaml = yaml_from_string(str);
        res.parse_yaml(yaml);
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest: Invalid YAML (%s)\n", ex.what());
    }
    return res;
}

void app_manifest_t::parse_yaml(const yaml_t& yaml)
{
    try
    {
        REQUIRED_TYPED_YAML_VALUE(yaml, app, _app);
        OPTIONAL_TYPED_YAML_VALUE(yaml, args, _args);
        REQUIRED_TYPED_YAML_VALUE(yaml, author, _author);
        OPTIONAL_TYPED_YAML_VALUE(yaml, avatar, _avatar);
        OPTIONAL_TYPED_YAML_VALUE(yaml, category, _category);

        OPTIONAL_YAML_NODE(yaml, conffiles, conffiles);
        for (const auto& conf : conffiles)
        {
            _conffiles.emplace_back(conffile_t{conf.as<std::string>()});
        }

        OPTIONAL_TYPED_YAML_VALUE(yaml, description, _description);
        OPTIONAL_YAML_NODE(yaml, devices, devices);
        for (const auto& device : devices)
        {
            _devices.emplace(device.as<std::string>());
        }
        OPTIONAL_TYPED_YAML_VALUE(yaml, editor, _editor);

        OPTIONAL_YAML_NODE(yaml, env, envs);
        for (const auto& env : envs)
        {
            _env.emplace(mapped_env_var_t{env.as<std::string>()});
        }

        OPTIONAL_TYPED_YAML_VALUE(yaml, hostname, _hostname);
        REQUIRED_TYPED_YAML_VALUE(yaml, image, _image);
        OPTIONAL_TYPED_YAML_VALUE(yaml, interactive, _interactive);
        OPTIONAL_TYPED_YAML_VALUE(yaml, multiInstance, _multi_instance);

        _networks.emplace_back("flecs");

        OPTIONAL_YAML_NODE(yaml, networkSettings, network_settings);
        for (const auto& setting : network_settings)
        {
            auto mac_address = std::string{};
            OPTIONAL_TYPED_YAML_VALUE(setting, macAddress, mac_address);
            if (!mac_address.empty())
            {
                _networks.rbegin()->mac_address(mac_address);
            }
        }

        OPTIONAL_YAML_NODE(yaml, ports, ports);
        for (const auto& port_range : ports)
        {
            _ports.emplace_back(mapped_port_range_t{port_range.as<std::string>()});
        }

        OPTIONAL_YAML_NODE(yaml, startupOptions, startup_options);
        for (const auto& startup_option : startup_options)
        {
            _startup_options.emplace_back(startup_option_from_string(startup_option.as<std::string>()));
        }

        REQUIRED_TYPED_YAML_VALUE(yaml, title, _title);
        REQUIRED_TYPED_YAML_VALUE(yaml, version, _version);

        OPTIONAL_YAML_NODE(yaml, volumes, volumes);
        for (const auto& volume : volumes)
        {
            _volumes.emplace_back(volume_t{volume.as<std::string>()});
        }

        _yaml_loaded = true;

        validate_yaml();
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest: Invalid YAML (%s)\n", ex.what());
        *this = app_manifest_t{};
    }
}

void app_manifest_t::validate_yaml()
{
    _yaml_valid = false;
    for (const auto& conffile : _conffiles)
    {
        if (!conffile.is_valid())
        {
            return;
        }
    }

    for (const auto& env : _env)
    {
        if (!env.is_valid())
        {
            return;
        }
    }

    for (const auto& port : _ports)
    {
        if (!port.is_valid())
        {
            return;
        }
    }

    for (const auto& volume : _volumes)
    {
        if (!volume.is_valid())
        {
            return;
        }
    }

    if (!_hostname.empty() && _multi_instance)
    {
        return;
    }

    _yaml_valid = true;
}

auto to_json(json_t& json, const app_manifest_t& app_manifest) //
    -> void
{
    json = json_t{
        {"app", app_manifest._app},
        {"args", app_manifest._args},
        {"author", app_manifest._author},
        {"avatar", app_manifest._avatar},
        {"category", app_manifest._category},
        {"conffiles", app_manifest._conffiles},
        {"description", app_manifest._description},
        {"devices", app_manifest._devices},
        {"editor", app_manifest._editor},
        {"env", app_manifest._env},
        {"hostname", app_manifest._hostname},
        {"image", app_manifest._image},
        {"interactive", app_manifest._interactive},
        {"multiInstance", app_manifest._multi_instance},
        {"networks", app_manifest._networks},
        {"ports", app_manifest._ports},
        {"title", app_manifest._title},
        {"version", app_manifest._version},
        {"volumes", app_manifest._volumes},
        {"yamlLoaded", app_manifest._yaml_loaded},
        {"yamlValid", app_manifest._yaml_valid},
    };
}

auto from_json(const json_t& json, app_manifest_t& app_manifest) //
    -> void
{
    json.at("app").get_to(app_manifest._app);
    json.at("args").get_to(app_manifest._args);
    json.at("author").get_to(app_manifest._author);
    json.at("avatar").get_to(app_manifest._avatar);
    json.at("category").get_to(app_manifest._category);
    json.at("conffiles").get_to(app_manifest._conffiles);
    json.at("description").get_to(app_manifest._description);
    json.at("devices").get_to(app_manifest._devices);
    json.at("editor").get_to(app_manifest._editor);
    json.at("env").get_to(app_manifest._env);
    json.at("hostname").get_to(app_manifest._hostname);
    json.at("image").get_to(app_manifest._image);
    json.at("interactive").get_to(app_manifest._interactive);
    json.at("multiInstance").get_to(app_manifest._multi_instance);
    json.at("networks").get_to(app_manifest._networks);
    json.at("ports").get_to(app_manifest._ports);
    json.at("title").get_to(app_manifest._title);
    json.at("version").get_to(app_manifest._version);
    json.at("volumes").get_to(app_manifest._volumes);
    json.at("yamlLoaded").get_to(app_manifest._yaml_loaded);
    json.at("yamlValid").get_to(app_manifest._yaml_valid);
}

} // namespace FLECS
