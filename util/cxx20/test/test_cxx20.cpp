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
#include "util/cxx20/string.h"

TEST(cxx20, string)
{
    using std::operator""s;
    using std::operator""sv;

    const auto s = "This is a string";
    const auto str = "This is a string"s;
    const auto sv = "This is a string"sv;

    ASSERT_TRUE(FLECS::cxx20::contains(s, "This"));
    ASSERT_TRUE(FLECS::cxx20::contains(str, "This"));
    ASSERT_TRUE(FLECS::cxx20::contains(sv, "This"));

    ASSERT_TRUE(FLECS::cxx20::contains(s, "string"sv));
    ASSERT_TRUE(FLECS::cxx20::contains(str, "string"sv));
    ASSERT_TRUE(FLECS::cxx20::contains(sv, "string"sv));

    ASSERT_FALSE(FLECS::cxx20::contains(s, "flecs"));
    ASSERT_FALSE(FLECS::cxx20::contains(str, "flecs"));
    ASSERT_FALSE(FLECS::cxx20::contains(sv, "flecs"));

    ASSERT_FALSE(FLECS::cxx20::contains(s, "123"sv));
    ASSERT_FALSE(FLECS::cxx20::contains(str, "123"sv));
    ASSERT_FALSE(FLECS::cxx20::contains(sv, "123"sv));
}
