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

#include <array>

#include "daemon/common/app/app_status.h"

TEST(app_status, to_string)
{
    const auto values = std::array<flecs::app_status_e, 10>{
        flecs::app_status_e::NotInstalled,
        flecs::app_status_e::ManifestDownloaded,
        flecs::app_status_e::TokenAcquired,
        flecs::app_status_e::ImageDownloaded,
        flecs::app_status_e::Installed,
        flecs::app_status_e::Removed,
        flecs::app_status_e::Purged,
        flecs::app_status_e::Orphaned,
        flecs::app_status_e::Unknown,
        static_cast<flecs::app_status_e>(-1),
    };

    const auto strings = std::array<std::string_view, 10>{
        "not installed",
        "manifest downloaded",
        "token acquired",
        "image downloaded",
        "installed",
        "removed",
        "purged",
        "orphaned",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(flecs::to_string(values[i]), strings[i]);
    }

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(flecs::to_string_view(values[i]), strings[i]);
    }

    /* skip last element as conversion is not bidirectional */
    for (size_t i = 0; i < values.size() - 1; ++i) {
        ASSERT_EQ(flecs::app_status_from_string(strings[i]), values[i]);
    }
}
