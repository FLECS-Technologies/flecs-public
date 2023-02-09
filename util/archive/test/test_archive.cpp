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

#include <array>
#include <fstream>
#include <string>
#include <string_view>

#include "gtest/gtest.h"
#include "util/archive/archive.h"
#include "util/fs/fs.h"

constexpr auto archives_supported = std::array<std::string_view, 3>{{
    "./archives/archive.tar",
    "./archives/archive.tar.gz",
    "./archives/archive.zip",
}};

constexpr auto archives_unsupported = std::array<std::string_view, 2>{{
    "archive.rar",    /* unsupported */
    "archive.tar.7z", /* unsupported */
}};

const auto files = std::array<FLECS::fs::path, 2>{{
    "compress/1/hello_flecs.txt",
    "compress/2/main.cpp",
}};

TEST(archive, init)
{
    for (const auto& file : files) {
        FLECS::fs::create_directories(file.parent_path());
    }

    {
        auto f = std::ofstream{files[0]};
        f << "Hello, FLECS!";
    }
    {
        auto f = std::ofstream{files[1]};
        f << "int main() { return 0; }";
    }
}

TEST(archive, compress_formats)
{
    for (const auto& archive : archives_supported) {
        const auto res = FLECS::archive::compress(archive, {{files[0]}, {files[1]}}, ".");
        ASSERT_EQ(res, 0);
        ASSERT_TRUE(FLECS::fs::is_regular_file(archive));

        const auto list = FLECS::archive::list(archive);
        ASSERT_EQ(list.size(), 2);
        ASSERT_EQ(list[0], files[0]);
        ASSERT_EQ(list[1], files[1]);

        FLECS::fs::remove(archive);
    }

    for (const auto& archive : archives_unsupported) {
        const auto res = FLECS::archive::compress(archive, {{files[0]}, {files[1]}}, ".");
        ASSERT_EQ(res, -1);
        ASSERT_FALSE(FLECS::fs::exists(archive));

        const auto list = FLECS::archive::list(archive);
        ASSERT_TRUE(list.empty());
    }
}

TEST(archive, compress_dir)
{
    const auto& archive = archives_supported[0];

    /* subdirectory of wd */
    {
        const auto res = FLECS::archive::compress(archive, {"./compress/1"}, "./compress");
        ASSERT_EQ(res, 0);
        ASSERT_TRUE(FLECS::fs::is_regular_file(archive));

        const auto list = FLECS::archive::list(archive);
        ASSERT_EQ(list.size(), 1);
        ASSERT_EQ(list[0], "1/hello_flecs.txt");

        FLECS::fs::remove(archive);
    }
    /* sibling directory of wd*/
    {
        const auto res = FLECS::archive::compress(archive, {"./compress/1"}, "./compress/2");
        ASSERT_EQ(res, 0);
        ASSERT_TRUE(FLECS::fs::is_regular_file(archive));

        const auto list = FLECS::archive::list(archive);
        ASSERT_EQ(list.size(), 1);
        ASSERT_EQ(list[0], "1/hello_flecs.txt");

        FLECS::fs::remove(archive);
    }
}

TEST(archive, compress_files_err)
{
    const auto& archive = archives_supported[0];

    /* file does not exist */
    {
        const auto res = FLECS::archive::compress(archive, {"./compress/3/nosuch.file"}, ".");
        ASSERT_EQ(res, -1);

        const auto list = FLECS::archive::list(archive);
        ASSERT_TRUE(list.empty());
    }
}

TEST(archive, compress_dir_err)
{
    const auto& archive = archives_supported[0];

    /* wd does not exist */
    {
        const auto res = FLECS::archive::compress(archive, {"./compress/1"}, "./compress/3");
        ASSERT_EQ(res, -1);

        const auto list = FLECS::archive::list(archive);
        ASSERT_TRUE(list.empty());
    }
}

TEST(archive, decompress)
{
    using std::operator""s;

    const auto& archive = archives_supported[0];

    FLECS::archive::compress(archive, {"./compress"}, ".");
    {
        const auto res = FLECS::archive::decompress(archive, "./decompress");

        {
            auto path = FLECS::fs::path{"./decompress"} / files[0];
            auto f = std::ifstream{path};
            auto str = std::string{};
            std::getline(f, str);
            ASSERT_EQ(str, "Hello, FLECS!");
        }
        {
            auto path = FLECS::fs::path{"./decompress"} / files[1];
            auto f = std::ifstream{path};
            auto str = std::string{};
            std::getline(f, str);
            ASSERT_EQ(str, "int main() { return 0; }");
        }

        ASSERT_EQ(res, 0);
    }
    FLECS::fs::remove_all("./decompress");
    FLECS::fs::remove(archive);

    FLECS::archive::compress(archive, {"./compress"}, "./compress");
    {
        const auto res = FLECS::archive::decompress(archive, "./decompress");
        ASSERT_EQ(res, 0);
    }
    FLECS::fs::remove_all("./decompress");
    FLECS::fs::remove(archive);
}

TEST(archive, teardown)
{
    FLECS::fs::remove_all("./compress");
}
