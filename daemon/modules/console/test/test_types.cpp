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

#include "daemon/modules/console/types.h"
#include "gtest/gtest.h"
#include "test_constants.h"

TEST(console, types)
{
    const auto uut = auth_json.get<flecs::console::auth_response_t>();

    ASSERT_EQ(uut.user().id(), 123);
    ASSERT_EQ(uut.user().user_email(), "user@flecs.tech");
    ASSERT_EQ(uut.user().user_login(), "user");
    ASSERT_EQ(uut.user().display_name(), "Some FLECS user");

    ASSERT_EQ(uut.jwt().token(), "eyJ0eXAiO...");
    ASSERT_EQ(uut.jwt().token_expires(), 1641034800);

    ASSERT_EQ(uut.feature_flags().is_vendor(), true);
    ASSERT_EQ(uut.feature_flags().is_white_labeled(), false);
}
