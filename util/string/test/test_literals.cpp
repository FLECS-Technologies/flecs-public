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

#include <string>

#include "gtest/gtest.h"
#include "util/string/literals.h"

TEST(string_utils, literals)
{
    using flecs::operator""_B;
    using flecs::operator""_kiB;
    using flecs::operator""_MiB;
    using flecs::operator""_GiB;
    using flecs::operator""_TiB;

    const auto B = 1_B;
    const auto kiB = 1_kiB;
    const auto MiB = 1_MiB;
    const auto GiB = 1_GiB;
    const auto TiB = 1_TiB;

    ASSERT_EQ(B, 1ULL);
    ASSERT_EQ(kiB, 1024ULL);
    ASSERT_EQ(MiB, 1048576ULL);
    ASSERT_EQ(GiB, 1073741824ULL);
    ASSERT_EQ(TiB, 1099511627776ULL);
}
