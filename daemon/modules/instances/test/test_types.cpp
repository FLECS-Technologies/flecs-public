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

#include <array>

#include "daemon/modules/instances/types.h"

TEST(instance_id, init)
{
    {
        const auto uut = flecs::instances::id_t{13};
        ASSERT_EQ(uut.get(), 13);
    }

    {
        const auto uut = flecs::instances::id_t{"01a55555"};
        ASSERT_EQ(uut.get(), 0x01a55555);
    }

    {
        const auto uut = flecs::instances::id_t{"invalid"};
        ASSERT_EQ(uut.get(), 0);
    }

    {
        const auto uut = flecs::instances::id_t{"1a2b3c4d5e6f"};
        ASSERT_EQ(uut.get(), 0);
    }
}

TEST(instance_id, regenerate)
{
    auto uut = flecs::instances::id_t{};
    const auto old_id = uut.get();
    uut.regenerate();
    ASSERT_NE(old_id, uut.get());
}

TEST(instance_id, hex)
{
    auto uut = flecs::instances::id_t{12648430};
    ASSERT_EQ(uut.hex(), "00c0ffee");
}

TEST(instance_id, compare)
{
    const auto uut1 = flecs::instances::id_t{2};
    const auto uut2 = flecs::instances::id_t{3};

    ASSERT_LT(uut1, uut2);
    ASSERT_LE(uut1, uut2);
    ASSERT_NE(uut1, uut2);
    ASSERT_GE(uut2, uut1);
    ASSERT_GT(uut2, uut1);

    ASSERT_EQ(uut1, uut1);
}

TEST(instance_status, to_string)
{
    const auto values = std::array<flecs::instances::status_e, 9>{
        flecs::instances::status_e::Created,
        flecs::instances::status_e::NotCreated,
        flecs::instances::status_e::Orphaned,
        flecs::instances::status_e::Requested,
        flecs::instances::status_e::ResourcesReady,
        flecs::instances::status_e::Running,
        flecs::instances::status_e::Stopped,
        flecs::instances::status_e::Unknown,
        static_cast<flecs::instances::status_e>(-1),
    };

    const auto strings = std::array<std::string_view, 9>{
        "created",
        "not created",
        "orphaned",
        "requested",
        "resources ready",
        "running",
        "stopped",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(to_string(values[i]), strings[i]);
    }

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(to_string_view(values[i]), strings[i]);
    }

    /* skip last element as conversion is not bidirectional */
    for (size_t i = 0; i < values.size() - 1; ++i) {
        ASSERT_EQ(flecs::instances::status_from_string(strings[i]), values[i]);
    }
}
