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

#include "yaml-cpp/yaml.h"

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
        {                                                 \
        }                                                 \
    } while (false)

#define OPTIONAL_YAML_VALUE(yaml, value, target) \
    do                                           \
    {                                            \
        try                                      \
        {                                        \
            target = yaml[#value];               \
        }                                        \
        catch (const YAML::Exception& ex)        \
        {                                        \
        }                                        \
    } while (false)

app_t::app_t(const std::string& manifest)
{
    try
    {
        const auto yaml = YAML::LoadFile(manifest);

        REQUIRED_TYPED_YAML_VALUE(yaml, app, _name);
        REQUIRED_TYPED_YAML_VALUE(yaml, title, _title);
        REQUIRED_TYPED_YAML_VALUE(yaml, version, _version);
        OPTIONAL_TYPED_YAML_VALUE(yaml, description, _description);
        REQUIRED_TYPED_YAML_VALUE(yaml, author, _author);
        OPTIONAL_TYPED_YAML_VALUE(yaml, category, _category);
        REQUIRED_TYPED_YAML_VALUE(yaml, image, _image);
        OPTIONAL_TYPED_YAML_VALUE(yaml, multiInstance, _multi_instance);
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
                return;
            }
            add_port(mapped_range);
        }
        OPTIONAL_TYPED_YAML_VALUE(yaml, interactive, _interactive);
        _yaml_loaded = true;
    }
    catch (const YAML::Exception& ex)
    {
        std::fprintf(stderr, "Could not open manifest %s: Invalid YAML (%s)\n", manifest.c_str(), ex.what());
    }
}

} // namespace FLECS
