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

#ifndef E8E3AE12_7249_481B_B47C_5682C1BBADE2
#define E8E3AE12_7249_481B_B47C_5682C1BBADE2

#include <string>
#include <unordered_map>
#include <vector>

namespace FLECS {

struct instance_config_t
{
    struct network_adapter_t
    {
        std::string name;
        std::string ipAddress;
        std::string subnetMask;
        std::string gateway;
        bool active;
    };
    std::vector<network_adapter_t> networkAdapters;

    struct network_t
    {
        std::string network;
        std::string ip;
    };
    std::vector<network_t> networks;

    std::vector<unsigned> startup_options;
};

} // namespace FLECS

#endif // E8E3AE12_7249_481B_B47C_5682C1BBADE2
