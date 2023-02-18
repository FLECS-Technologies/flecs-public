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

#include "daemon/common/app/manifest/startup_option/startup_option.h"

TEST(startup_option, valid)
{
    const auto str = "initNetworkAfterStart";
    const auto opt = FLECS::startup_option_from_string(str);

    ASSERT_EQ(opt, FLECS::startup_option_t::INIT_NETWORK_AFTER_START);
}

TEST(startup_option, invalid)
{
    const auto str = "invalidStartupOption";
    const auto opt = FLECS::startup_option_from_string(str);

    ASSERT_EQ(opt, FLECS::startup_option_t::INVALID);
}
