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

#include <gtest/gtest.h>

#include "flecs/common/network/network_type.h"

TEST(network_type, to_string)
{
    const auto to_string_mapping = std::array<std::pair<flecs::network_type_e, std::string_view>, 8>{{
        {flecs::network_type_e::None, "none"},
        {flecs::network_type_e::Internal, "internal"},
        {flecs::network_type_e::Bridge, "bridge"},
        {flecs::network_type_e::MACVLAN, "macvlan"},
        {flecs::network_type_e::IPVLAN_L2, "ipvlan_l2"},
        {flecs::network_type_e::IPVLAN_L3, "ipvlan_l3"},
        {flecs::network_type_e::Unknown, "unknown"},
        {static_cast<flecs::network_type_e>(-1), "unknown"},
    }};

    const auto from_string_mapping = std::array<std::pair<std::string_view, flecs::network_type_e>, 8>{{
        {"none", flecs::network_type_e::None},
        {"internal", flecs::network_type_e::Internal},
        {"bridge", flecs::network_type_e::Bridge},
        {"macvlan", flecs::network_type_e::MACVLAN},
        {"ipvlan", flecs::network_type_e::IPVLAN_L2},
        {"ipvlan_l2", flecs::network_type_e::IPVLAN_L2},
        {"ipvlan_l3", flecs::network_type_e::IPVLAN_L3},
        {"unknown", flecs::network_type_e::Unknown},
    }};

    for (const auto& uut : to_string_mapping) {
        ASSERT_EQ(flecs::to_string(uut.first), uut.second);
        ASSERT_EQ(flecs::to_string_view(uut.first), uut.second);
    }

    for (const auto& uut : from_string_mapping) {
        ASSERT_EQ(flecs::network_type_from_string(uut.first), uut.second);
    }
}
