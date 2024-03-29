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
    const auto values = std::array<flecs::network_type_e, 7>{
        flecs::network_type_e::None,
        flecs::network_type_e::Internal,
        flecs::network_type_e::Bridge,
        flecs::network_type_e::MACVLAN,
        flecs::network_type_e::IPVLAN,
        flecs::network_type_e::Unknown,
        static_cast<flecs::network_type_e>(-1),
    };

    const auto strings = std::array<std::string_view, 9>{
        "none",
        "internal",
        "bridge",
        "macvlan",
        "ipvlan",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(flecs::to_string(values[i]), strings[i]);
    }

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(flecs::to_string_view(values[i]), strings[i]);
    }

    /* skip last element as conversion is not bidirectional */
    for (size_t i = 0; i < values.size() - 1; ++i) {
        ASSERT_EQ(flecs::network_type_from_string(strings[i]), values[i]);
    }
}
