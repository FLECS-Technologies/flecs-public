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

#include <gtest/gtest.h>

#include "daemon/common/app/manifest/conffile/conffile.h"

TEST(conffile, empty)
{
    const auto conffile = FLECS::conffile_t{};

    ASSERT_FALSE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "");
    ASSERT_EQ(conffile.container(), "");
    ASSERT_FALSE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}

TEST(conffile, valid)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}

TEST(conffile, invalid_mapping_1)
{
    const auto conffile = FLECS::conffile_t{std::string{"a"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_mapping_2)
{
    const auto conffile = FLECS::conffile_t{std::string{"a:"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_mapping_3)
{
    const auto conffile = FLECS::conffile_t{std::string{":a"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_local_path)
{
    const auto conffile = FLECS::conffile_t{std::string{"/path/to/file.cfg:/etc/file.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_local_char)
{
    const auto conffile = FLECS::conffile_t{std::string{"file*.cfg:/etc/file.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_path_1)
{
    const auto conffile = FLECS::conffile_t{std::string{"file*.cfg:/etc/conf.d/"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_path_2)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:conf.d/"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_char)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/conf.d/file*.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, valid_properties_1)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:ro,init"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_TRUE(conffile.ro());
    ASSERT_TRUE(conffile.init());
}

TEST(conffile, valid_properties_2)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:init,ro"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_TRUE(conffile.ro());
    ASSERT_TRUE(conffile.init());
}

TEST(conffile, valid_properties_3)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:rw,no_init"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}

TEST(conffile, invalid_properties_1)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}

TEST(conffile, invalid_properties_2)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop,another_invalid_prop"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}

TEST(conffile, invalid_properties_3)
{
    const auto conffile = FLECS::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop,ro"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_TRUE(conffile.ro());
    ASSERT_FALSE(conffile.init());
}
