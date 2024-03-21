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

#include <string>

#include "flecs/util/string/format.h"

TEST(string_utils, format_uint32)
{
    using namespace flecs;

    const std::uint32_t i = 15;

    const auto str_1 = int_to_hex(i, fmt::Lowercase, fmt::NoPrefix, fmt::NoLeadingZeroes);
    const auto str_2 = int_to_hex(i, fmt::Lowercase, fmt::NoPrefix, fmt::LeadingZeroes);
    const auto str_3 = int_to_hex(i, fmt::Lowercase, fmt::Prefix, fmt::NoLeadingZeroes);
    const auto str_4 = int_to_hex(i, fmt::Lowercase, fmt::Prefix, fmt::LeadingZeroes);
    const auto str_5 = int_to_hex(i, fmt::Uppercase, fmt::NoPrefix, fmt::NoLeadingZeroes);
    const auto str_6 = int_to_hex(i, fmt::Uppercase, fmt::NoPrefix, fmt::LeadingZeroes);
    const auto str_7 = int_to_hex(i, fmt::Uppercase, fmt::Prefix, fmt::NoLeadingZeroes);
    const auto str_8 = int_to_hex(i, fmt::Uppercase, fmt::Prefix, fmt::LeadingZeroes);

    ASSERT_EQ(str_1, "f");
    ASSERT_EQ(str_2, "0000000f");
    ASSERT_EQ(str_3, "0xf");
    ASSERT_EQ(str_4, "0x0000000f");
    ASSERT_EQ(str_5, "F");
    ASSERT_EQ(str_6, "0000000F");
    ASSERT_EQ(str_7, "0XF");
    ASSERT_EQ(str_8, "0X0000000F");
}

TEST(string_utils, format_uint16)
{
    using namespace flecs;

    const std::uint16_t i = 15;

    const auto str_1 = int_to_hex(i, fmt::Lowercase, fmt::NoPrefix, fmt::NoLeadingZeroes);
    const auto str_2 = int_to_hex(i, fmt::Lowercase, fmt::NoPrefix, fmt::LeadingZeroes);
    const auto str_3 = int_to_hex(i, fmt::Lowercase, fmt::Prefix, fmt::NoLeadingZeroes);
    const auto str_4 = int_to_hex(i, fmt::Lowercase, fmt::Prefix, fmt::LeadingZeroes);
    const auto str_5 = int_to_hex(i, fmt::Uppercase, fmt::NoPrefix, fmt::NoLeadingZeroes);
    const auto str_6 = int_to_hex(i, fmt::Uppercase, fmt::NoPrefix, fmt::LeadingZeroes);
    const auto str_7 = int_to_hex(i, fmt::Uppercase, fmt::Prefix, fmt::NoLeadingZeroes);
    const auto str_8 = int_to_hex(i, fmt::Uppercase, fmt::Prefix, fmt::LeadingZeroes);

    ASSERT_EQ(str_1, "f");
    ASSERT_EQ(str_2, "000f");
    ASSERT_EQ(str_3, "0xf");
    ASSERT_EQ(str_4, "0x000f");
    ASSERT_EQ(str_5, "F");
    ASSERT_EQ(str_6, "000F");
    ASSERT_EQ(str_7, "0XF");
    ASSERT_EQ(str_8, "0X000F");
}
