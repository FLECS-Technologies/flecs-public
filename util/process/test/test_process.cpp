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

#include "gtest/gtest.h"
#include "util/process/process.h"

TEST(util_process, spawn)
{
    auto test_process = flecs::process_t{};

    const auto spawn_res = test_process.spawn("/bin/echo", "-n", "FLECS");
    ASSERT_EQ(spawn_res, 0);

    const auto wait_res = test_process.wait(false, false);
    ASSERT_NE(wait_res, -1);

    ASSERT_EQ(test_process.exit_code(), 0);
    ASSERT_EQ(test_process.stdout(), "FLECS");
}

TEST(util_process, spawnp)
{
    auto test_process = flecs::process_t{};

    const auto spawn_res = test_process.spawnp("echo", "-n", "FLECS");
    ASSERT_EQ(spawn_res, 0);

    const auto wait_res = test_process.wait(false, false);
    ASSERT_NE(wait_res, -1);

    ASSERT_EQ(test_process.exit_code(), 0);
    ASSERT_EQ(test_process.stdout(), "FLECS");
}

TEST(util_process, spawnp_args)
{
    auto test_process = flecs::process_t{};

    test_process.arg("-n");
    test_process.arg("FLECS");

    testing::internal::CaptureStdout();

    const auto spawn_res = test_process.spawnp("echo");
    ASSERT_EQ(spawn_res, 0);

    const auto wait_res = test_process.wait(true, true);
    ASSERT_NE(wait_res, -1);

    ASSERT_EQ(test_process.exit_code(), 0);
    ASSERT_EQ(testing::internal::GetCapturedStdout(), "FLECS");
}

TEST(util_process, spawn_fail)
{
    auto test_process = flecs::process_t{};

    const auto spawn_res = test_process.spawnp("nonexistent-binary");
    ASSERT_NE(spawn_res, 0);

    const auto wait_res = test_process.wait(false, false);
    ASSERT_EQ(wait_res, -1);

    ASSERT_EQ(test_process.stdout(), "");
    ASSERT_EQ(test_process.stderr(), "");
}

TEST(util_process, spawnp_fail)
{
    auto test_process = flecs::process_t{};

    const auto spawn_res = test_process.spawn("/this/path/does/not/exist");
    ASSERT_NE(spawn_res, 0);

    const auto wait_res = test_process.wait(false, false);
    ASSERT_EQ(wait_res, -1);

    ASSERT_EQ(test_process.stdout(), "");
    ASSERT_EQ(test_process.stderr(), "");
}

TEST(util_process, move_construct)
{
    auto test_process = flecs::process_t{};

    test_process.arg("-n");
    test_process.arg("FLECS");

    auto test_process_2 = flecs::process_t{std::move(test_process)};

    const auto spawn_res = test_process_2.spawnp("echo");
    ASSERT_EQ(spawn_res, 0);

    const auto wait_res = test_process_2.wait(false, false);
    ASSERT_NE(wait_res, -1);

    ASSERT_EQ(test_process_2.exit_code(), 0);
    ASSERT_EQ(test_process_2.stdout(), "FLECS");
}

TEST(util_process, assign)
{
    auto test_process = flecs::process_t{};

    test_process.arg("-n");
    test_process.arg("FLECS");

    test_process = flecs::process_t{};

    const auto spawn_res = test_process.spawnp("echo");
    ASSERT_EQ(spawn_res, 0);

    const auto wait_res = test_process.wait(false, false);
    ASSERT_NE(wait_res, -1);

    ASSERT_EQ(test_process.exit_code(), 0);
    ASSERT_EQ(test_process.stdout(), "\n");
}
