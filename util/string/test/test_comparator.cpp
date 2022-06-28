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

#include <string>

#include "gtest/gtest.h"
#include "util/string/comparator.h"

TEST(string_comparator, compare)
{
    const auto str1 = "abc";
    const auto str2 = "def";
    const auto str3 = "abcdef";

    const auto comparator = FLECS::string_comparator_t{};

    ASSERT_TRUE(comparator(str1, str2));
    ASSERT_TRUE(comparator(str1, str3));
    ASSERT_TRUE(comparator(str3, str2));
    ASSERT_FALSE(comparator(str1, str1));
}
