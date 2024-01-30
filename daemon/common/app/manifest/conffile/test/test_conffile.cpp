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

#include "daemon/common/app/manifest/conffile/conffile.h"

TEST(conffile, empty)
{
    const auto conffile = flecs::conffile_t{};

    ASSERT_FALSE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "");
    ASSERT_EQ(conffile.container(), "");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, valid)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, invalid_mapping_1)
{
    const auto conffile = flecs::conffile_t{std::string{"a"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_mapping_2)
{
    const auto conffile = flecs::conffile_t{std::string{"a:"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_mapping_3)
{
    const auto conffile = flecs::conffile_t{std::string{":a"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_local_path)
{
    const auto conffile = flecs::conffile_t{std::string{"/path/to/file.cfg:/etc/file.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_local_char)
{
    const auto conffile = flecs::conffile_t{std::string{"file*.cfg:/etc/file.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_path_1)
{
    const auto conffile = flecs::conffile_t{std::string{"file*.cfg:/etc/conf.d/"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_path_2)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:conf.d/"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, invalid_container_char)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/conf.d/file*.cfg"}};

    ASSERT_FALSE(conffile.is_valid());
}

TEST(conffile, valid_properties_1)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:ro"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_TRUE(conffile.ro());
}

TEST(conffile, valid_properties_2)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, valid_properties_3)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:rw"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, invalid_properties_1)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, invalid_properties_2)
{
    const auto conffile =
        flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop,another_invalid_prop"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_FALSE(conffile.ro());
}

TEST(conffile, invalid_properties_3)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop,ro"}};

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_TRUE(conffile.ro());
}

TEST(conffile, to_json)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg"}};

    const auto json = flecs::json_t(conffile);
    const auto json_expected = R"("file.cfg:/etc/file.cfg:rw")";

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(json.dump(), json_expected);
}

TEST(conffile, from_json)
{
    const auto json_string = R"("file.cfg:/etc/file.cfg:rw")";
    const auto json = flecs::parse_json(json_string);

    const auto conffile = json.get<flecs::conffile_t>();

    ASSERT_TRUE(conffile.is_valid());
    ASSERT_EQ(conffile.container(), "/etc/file.cfg");
    ASSERT_EQ(conffile.local(), "file.cfg");
    ASSERT_EQ(conffile.ro(), false);
}

TEST(conffile, sort)
{
    const auto uut_1 = flecs::conffile_t{"file.cfg:/etc/file.cfg:ro"};
    const auto uut_2 = flecs::conffile_t{"another_file.cfg:/etc/file2.cfg:rw"};

    ASSERT_LT(uut_2, uut_1);
    ASSERT_LE(uut_2, uut_1);
    ASSERT_NE(uut_2, uut_1);
    ASSERT_GE(uut_1, uut_2);
    ASSERT_GT(uut_1, uut_2);
}

TEST(conffile, to_string)
{
    const auto conffile = flecs::conffile_t{std::string{"file.cfg:/etc/file.cfg:invalid_prop,ro"}};

    const auto expected = "file.cfg:/etc/file.cfg:ro";

    ASSERT_EQ(to_string(conffile), expected);
}
