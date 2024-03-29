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

#include "flecs/common/app/manifest/volume/volume.h"

TEST(volume, volume)
{
    const auto volume_1 = flecs::volume_t{"mnt:/path/to/mnt"};

    ASSERT_TRUE(volume_1.is_valid());
    ASSERT_EQ(volume_1.host(), "mnt");
    ASSERT_EQ(volume_1.container(), "/path/to/mnt");
    ASSERT_EQ(volume_1.type(), flecs::volume_t::VOLUME);

    const auto volume_2 = flecs::volume_t{"another_mnt:/path/to/another/mnt"};

    ASSERT_TRUE(volume_2.is_valid());
    ASSERT_EQ(volume_2.host(), "another_mnt");
    ASSERT_EQ(volume_2.container(), "/path/to/another/mnt");
    ASSERT_EQ(volume_2.type(), flecs::volume_t::VOLUME);

    const auto volume_3 = flecs::volume_t{"invalid$mnt:/path/to/invalid/mnt"};

    ASSERT_FALSE(volume_3.is_valid());
    ASSERT_EQ(volume_3.host(), "");
    ASSERT_EQ(volume_3.container(), "");
    ASSERT_EQ(volume_3.type(), flecs::volume_t::NONE);

    const auto volume_4 = flecs::volume_t{"mnt:path/to/invalid/mnt"};

    ASSERT_FALSE(volume_4.is_valid());
    ASSERT_EQ(volume_4.host(), "");
    ASSERT_EQ(volume_4.container(), "");
    ASSERT_EQ(volume_4.type(), flecs::volume_t::NONE);
}

TEST(volume, bind_mount)
{
    const auto bind_mount_1 = flecs::volume_t{"/path/to/host:/path/to/container"};

    ASSERT_TRUE(bind_mount_1.is_valid());
    ASSERT_EQ(bind_mount_1.host(), "/path/to/host");
    ASSERT_EQ(bind_mount_1.container(), "/path/to/container");
    ASSERT_EQ(bind_mount_1.type(), flecs::volume_t::BIND_MOUNT);

    const auto bind_mount_2 = flecs::volume_t{"invalid/path/to/host:/path/to/container"};

    ASSERT_FALSE(bind_mount_2.is_valid());
    ASSERT_EQ(bind_mount_2.host(), "");
    ASSERT_EQ(bind_mount_2.container(), "");
    ASSERT_EQ(bind_mount_2.type(), flecs::volume_t::NONE);
}

TEST(volume, invalid)
{
    const auto invalid_1 = flecs::volume_t{"invalid"};

    ASSERT_FALSE(invalid_1.is_valid());
    ASSERT_EQ(invalid_1.host(), "");
    ASSERT_EQ(invalid_1.container(), "");
    ASSERT_EQ(invalid_1.type(), flecs::volume_t::NONE);
}

TEST(volume, to_json)
{
    const auto volume_1 = flecs::volume_t{"mnt:/path/to/mnt"};
    const auto json_1 = flecs::json_t(volume_1);
    const auto expected_1 = R"("mnt:/path/to/mnt")";

    ASSERT_EQ(json_1.dump(), expected_1);

    const auto bind_mount_1 = flecs::volume_t{"/path/to/host:/path/to/container"};
    const auto json_2 = flecs::json_t(bind_mount_1);
    const auto expected_2 = R"("/path/to/host:/path/to/container")";

    ASSERT_EQ(json_2.dump(), expected_2);

    const auto invalid_1 = flecs::volume_t{"invalid"};
    const auto json_3 = flecs::json_t(invalid_1);
    const auto expected_3 = R"("")";

    ASSERT_EQ(json_3.dump(), expected_3);
}
