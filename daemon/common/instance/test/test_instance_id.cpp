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

#include "daemon/common/instance/instance_id.h"

TEST(instance_id, init)
{
    {
        const auto uut = FLECS::instance_id_t{13};
        ASSERT_EQ(uut.get(), 13);
    }

    {
        const auto uut = FLECS::instance_id_t{"01a55555"};
        ASSERT_EQ(uut.get(), 0x01a55555);
    }

    {
        const auto uut = FLECS::instance_id_t{"invalid"};
        ASSERT_EQ(uut.get(), 0);
    }

    {
        const auto uut = FLECS::instance_id_t{"1a2b3c4d5e6f"};
        ASSERT_EQ(uut.get(), 0);
    }
}

TEST(instance_id, regenerate)
{
    auto uut = FLECS::instance_id_t{};
    const auto old_id = uut.get();
    uut.regenerate();
    ASSERT_NE(old_id, uut.get());
}

TEST(instance_id, hex)
{
    auto uut = FLECS::instance_id_t{12648430};
    ASSERT_EQ(uut.hex(), "00c0ffee");
}

TEST(instance_id, compare)
{
    const auto uut1 = FLECS::instance_id_t{2};
    const auto uut2 = FLECS::instance_id_t{3};

    ASSERT_LT(uut1, uut2);
    ASSERT_LE(uut1, uut2);
    ASSERT_NE(uut1, uut2);
    ASSERT_GE(uut2, uut1);
    ASSERT_GT(uut2, uut1);

    ASSERT_EQ(uut1, uut1);
}
