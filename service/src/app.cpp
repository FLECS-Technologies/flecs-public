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

#include "service/app.h"

#include <iostream>
#include <sstream>
#include <vector>

#include "external/yaml-cpp-0.7.0/include/yaml-cpp/yaml.h"
#include "util/string/string_utils.h"

namespace FLECS {

app_t::app_t(const std::string& manifest)
{
    try
    {
        const auto yaml = YAML::LoadFile(manifest);

        _name = yaml["app"].as<std::string>();
        _version = yaml["version"].as<std::string>();
        _description = yaml["description"].as<std::string>();
        _author = yaml["author"].as<std::string>();
        _category = yaml["category"].as<std::string>();

        _image = yaml["image"].as<std::string>();
        _multi_instance = yaml["multiInstance"].as<bool>();
        for (const auto& i : yaml["volumes"])
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
        for (const auto& i : yaml["networks"])
        {
            const auto network = i.as<std::string>();
            add_network(network);
        }
        for (const auto& i : yaml["ports"])
        {
            const auto port = split(i.as<std::string>(), ':');
            add_port(std::stoi(port[0]), std::stoi(port[1]));
        }
        _yaml_loaded = true;
    } catch (const YAML::Exception& e)
    {
        std::cerr << "Could not load " << manifest << ": " << e.what() << std::endl;
    }
}

} // namespace FLECS
