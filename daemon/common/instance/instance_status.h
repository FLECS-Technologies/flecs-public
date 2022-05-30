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

#ifndef A1671630_FCCE_4302_B252_4C67A6FF0562
#define A1671630_FCCE_4302_B252_4C67A6FF0562

namespace FLECS {

enum instance_status_e : char
{
    NOT_CREATED = 'n',
    REQUESTED = 'q',
    RESOURCES_READY = 'y',
    CREATED = 'c',
    STOPPED = 's',
    RUNNING = 'r',
};

inline std::string to_string(instance_status_e val)
{
    auto res = std::string{};
    return res.append(1, val);
}

using instance_status_to_string_t = map_c<instance_status_e, const char*, 6>;
constexpr instance_status_to_string_t instance_status_to_string_table = {{
    std::make_pair(instance_status_e::NOT_CREATED, "not created"),
    std::make_pair(instance_status_e::REQUESTED, "requested"),
    std::make_pair(instance_status_e::RESOURCES_READY, "resources ready"),
    std::make_pair(instance_status_e::CREATED, "created"),
    std::make_pair(instance_status_e::STOPPED, "stopped"),
    std::make_pair(instance_status_e::RUNNING, "running"),
}};

constexpr const char* instance_status_to_string(instance_status_e status)
{
    if (instance_status_to_string_table.find(status) != instance_status_to_string_table.end())
    {
        return instance_status_to_string_table.at(status).second;
    }

    return "unknown";
}

} // namespace FLECS

#endif // A1671630_FCCE_4302_B252_4C67A6FF0562
