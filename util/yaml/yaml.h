// Copyright 2021-2023 FLECS Technologies GmbH
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

#pragma once

#include <yaml-cpp/yaml.h>

namespace flecs {

using yaml_t = YAML::Node;

inline auto yaml_from_string(std::string_view str)
{
    return YAML::Load(str.data());
}

inline auto yaml_from_file(std::string_view path)
{
    return YAML::LoadFile(path.data());
}

} // namespace flecs
