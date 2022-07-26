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

#include "instance_config.h"

namespace FLECS {

#if 0
auto to_json(json_t& json, const instance_config_t& instance_config) //
    -> void
{
    json = json_t{
        {"networkAdapters", instance_config.networkAdapters},
    }; //{"networks", instance_config.networks},
       //   {"startupOptions", instance_config.startup_options}};
}

auto from_json(const json_t& json, instance_config_t& instance_config) //
    -> void
{}
#endif // 0

} // namespace FLECS
