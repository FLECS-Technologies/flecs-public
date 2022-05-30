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

#ifndef B120BAAC_A4C6_4736_BCBD_04477ACD982F
#define B120BAAC_A4C6_4736_BCBD_04477ACD982F

#include <yaml-cpp/yaml.h>

namespace FLECS {

using yaml_t = YAML::Node;

inline auto yaml_from_string(const std::string& str)
{
    return YAML::Load(str);
}

inline auto yaml_from_file(const std::string& path)
{
    return YAML::LoadFile(path);
}

} // namespace FLECS

#endif // B120BAAC_A4C6_4736_BCBD_04477ACD982F
