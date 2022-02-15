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

#include "gtest/gtest.h"
#include "version/version.h"

TEST(module_version, print_version)
{
    testing::internal::CaptureStdout();
    testing::internal::CaptureStderr();

    const auto stdout_expected = std::string{FLECS_VERSION} + "-" + FLECS_GIT_SHA + "\n";
    const auto stderr_expected = std::string{};

    auto mod = FLECS::module_version_t{};
    const auto res = mod.process(0, nullptr);
    const auto stdout_actual = testing::internal::GetCapturedStdout();
    const auto stderr_actual = testing::internal::GetCapturedStderr();

    ASSERT_EQ(res, FLECS::module_error_e::FLECS_OK);
    ASSERT_EQ(stdout_actual, stdout_expected);
    ASSERT_EQ(stderr_actual, stderr_expected);
}
