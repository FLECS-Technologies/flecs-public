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

#include "daemon/common/instance/instance_status.h"

TEST(instance_status, to_string)
{
    const auto values = std::array<FLECS::instance_status_e, 9>{
        FLECS::instance_status_e::Created,
        FLECS::instance_status_e::NotCreated,
        FLECS::instance_status_e::Orphaned,
        FLECS::instance_status_e::Requested,
        FLECS::instance_status_e::ResourcesReady,
        FLECS::instance_status_e::Running,
        FLECS::instance_status_e::Stopped,
        FLECS::instance_status_e::Unknown,
        static_cast<FLECS::instance_status_e>(-1),
    };

    const auto strings = std::array<std::string_view, 9>{
        "created",
        "not created",
        "orphaned",
        "requested",
        "resources ready",
        "running",
        "stopped",
        "unknown",
        "unknown",
    };

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(FLECS::to_string(values[i]), strings[i]);
    }

    for (size_t i = 0; i < values.size(); ++i) {
        ASSERT_EQ(FLECS::to_string_view(values[i]), strings[i]);
    }

    /* skip last element as conversion is not bidirectional */
    for (size_t i = 0; i < values.size() - 1; ++i) {
        ASSERT_EQ(FLECS::instance_status_from_string(strings[i]), values[i]);
    }
}