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

#include "app.h"

#include <iostream>
#include <sstream>
#include <vector>

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

#define OPTIONAL_YAML_VALUE(yaml, value, target) \
    do                                           \
    {                                            \
        try                                      \
        {                                        \
            target = yaml[#value];               \
        }                                        \
        catch (const YAML::Exception& ex)        \
        {}                                       \
    } while (false)

app_t::app_t() noexcept
    : _yaml_loaded{}
    , _name{}
    , _title{}
    , _version{}
    , _description{}
    , _author{}
    , _category{}
    , _image{}
    , _env{}
    , _conffiles{}
    , _volumes{}
    , _bind_mounts{}
    , _hostname{}
    , _networks{}
    , _ports{}
    , _args{}
    , _interactive{}
    , _installed_size{}
    , _multi_instance{}
    , _status{}
    , _desired{}
{}

app_t app_t::from_file(const std::filesystem::path& path)
{
    auto res = app_t{};
    try
    {
        const auto yaml_node = YAML::LoadFile(path);
        res.load_yaml(yaml_node);
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest %s: Invalid YAML (%s)\n", path.c_str(), ex.what());
    }
    return res;
}

app_t app_t::from_string(const std::string& yaml)
{
    auto res = app_t{};
    try
    {
        const auto yaml_node = YAML::Load(yaml.c_str());
        res.load_yaml(yaml_node);
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest: Invalid YAML (%s)\n", ex.what());
    }
    return res;
}

void app_t::load_yaml(const YAML::Node& yaml)
{
    try
    {
        REQUIRED_TYPED_YAML_VALUE(yaml, app, _name);
        REQUIRED_TYPED_YAML_VALUE(yaml, title, _title);
        REQUIRED_TYPED_YAML_VALUE(yaml, version, _version);
        OPTIONAL_TYPED_YAML_VALUE(yaml, description, _description);
        REQUIRED_TYPED_YAML_VALUE(yaml, author, _author);
        OPTIONAL_TYPED_YAML_VALUE(yaml, category, _category);
        REQUIRED_TYPED_YAML_VALUE(yaml, image, _image);
        OPTIONAL_TYPED_YAML_VALUE(yaml, multiInstance, _multi_instance);

        auto envs = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, env, envs);
        for (const auto env : envs)
        {
            const auto env_var = mapped_env_var_t{env.as<std::string>()};
            if (!env_var.is_valid())
            {
                std::fprintf(
                    stderr,
                    "Could not parse manifest: syntax/schema error in %s\n",
                    env.as<std::string>().c_str());
                throw std::runtime_error{env.as<std::string>()};
            }
            add_env(env_var);
        }

        auto conffiles = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, conffiles, conffiles);
        for (const auto& conf : conffiles)
        {
            const auto conffile = conffile_t{conf.as<std::string>()};
            if (!conffile.is_valid())
            {
                std::fprintf(
                    stderr,
                    "Could not parse manifest: syntax/schema error in %s\n",
                    conf.as<std::string>().c_str());
                throw std::runtime_error{conf.as<std::string>()};
            }
            add_conffile(conffile);
        }

        auto volumes = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, volumes, volumes);
        for (const auto& i : volumes)
        {
            const auto volume = split(i.as<std::string>(), ':');
            if (cxx20::starts_with(volume[0], '/'))
            {
                add_bind_mount(volume[0], volume[1]);
            }
            else
            {
                add_volume(volume[0], volume[1]);
            }
        }
        OPTIONAL_TYPED_YAML_VALUE(yaml, hostname, _hostname);
        add_network("flecs");
        auto networks = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, networks, networks);
        for (const auto& i : networks)
        {
            const auto network = i.as<std::string>();
            add_network(network);
        }
        auto ports = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, ports, ports);
        for (const auto& port_range : ports)
        {
            const auto mapped_range = mapped_port_range_t{port_range.as<std::string>()};
            if (!mapped_range.is_valid())
            {
                std::fprintf(
                    stderr,
                    "Could not parse manifest: syntax/schema error in %s\n",
                    port_range.as<std::string>().c_str());
                throw std::runtime_error{port_range.as<std::string>()};
            }
            add_port(mapped_range);
        }

        auto args = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, args, args);
        for (const auto& arg : args)
        {
            add_arg(arg.as<std::string>());
        }
        OPTIONAL_TYPED_YAML_VALUE(yaml, interactive, _interactive);

        if (!_hostname.empty() && _multi_instance)
        {
            std::fprintf(stderr, "Could not load manifest: hostname is set alongside multi-instance\n");
            throw std::runtime_error{"hostname is set alongside multi-instance"};
        }
        _yaml_loaded = true;
    }
    catch (const std::exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest: Invalid YAML (%s)\n", ex.what());
        *this = app_t{};
    }
}

} // namespace FLECS
