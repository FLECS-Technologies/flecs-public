// Copyright 2021 FLECS Technologies GmbH
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

#include "app/app.h"

#include <iostream>
#include <sstream>
#include <vector>

#include "util/string/string_utils.h"
#include "yaml-cpp/yaml.h"

namespace FLECS {

#define REQUIRED_TYPED_YAML_VALUE(yaml, value, target, type) \
    do                                                       \
    {                                                        \
        target = yaml[#value].as<type>();                    \
    } while (false)

#define REQUIRED_YAML_VALUE(yaml, value, target) \
    do                                           \
    {                                            \
        target = yaml[#value];                   \
    } while (false)

#define OPTIONAL_TYPED_YAML_VALUE(yaml, value, target, type) \
    do                                                       \
    {                                                        \
        try                                                  \
        {                                                    \
            target = yaml[#value].as<type>();                \
        } catch (const YAML::Exception& ex)                  \
        {                                                    \
        }                                                    \
    } while (false)

#define OPTIONAL_YAML_VALUE(yaml, value, target) \
    do                                           \
    {                                            \
        try                                      \
        {                                        \
            target = yaml[#value];               \
        } catch (const YAML::Exception& ex)      \
        {                                        \
        }                                        \
    } while (false)

app_t::app_t(const std::string& manifest)
{
    const auto yaml = YAML::LoadFile(manifest);

    try
    {
        REQUIRED_TYPED_YAML_VALUE(yaml, app, _name, std::string);
        REQUIRED_TYPED_YAML_VALUE(yaml, title, _title, std::string);
        REQUIRED_TYPED_YAML_VALUE(yaml, version, _version, std::string);
        OPTIONAL_TYPED_YAML_VALUE(yaml, description, _description, std::string);
        REQUIRED_TYPED_YAML_VALUE(yaml, author, _author, std::string);
        OPTIONAL_TYPED_YAML_VALUE(yaml, category, _category, std::string);
        REQUIRED_TYPED_YAML_VALUE(yaml, image, _image, std::string);
        OPTIONAL_TYPED_YAML_VALUE(yaml, multiInstance, _multi_instance, bool);
        auto volumes = YAML::Node{};
        OPTIONAL_YAML_VALUE(yaml, volumes, volumes);
        for (const auto& i : volumes)
        {
            const auto volume = split(i.as<std::string>(), ':');
            if (cxx20::starts_with(volume[0], '/'))
            {
                add_bind_mount(volume[0], volume[1]);
            } else
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
        for (const auto& i : ports)
        {
            const auto port = split(i.as<std::string>(), ':');
            add_port(std::stoi(port[0]), std::stoi(port[1]));
        }
        _yaml_loaded = true;
    } catch (const YAML::Exception& ex)
    {
        std::fprintf(stderr, "%s\n", ex.what());
    }
}

} // namespace FLECS
