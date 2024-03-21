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

#include <thread>

#include "flecs/util/udev/udev.h"

TEST(udev, init)
{
    auto udev_1 = flecs::udev::udev_t{};
    auto udev_2 = flecs::udev::udev_t{};
    auto udev_3 = flecs::udev::udev_t{};

    ASSERT_NO_THROW((udev_2 = udev_1));
    ASSERT_NO_THROW((udev_2 = flecs::udev::udev_t{udev_1}));
    ASSERT_NO_THROW((udev_2 = flecs::udev::udev_t{std::move(udev_1)}));
    ASSERT_NO_THROW((udev_3 = std::move(udev_2)));
}

TEST(udev, multithreading)
{
    auto udev_1 = flecs::udev::udev_t{};

    auto t1 = std::thread{[&]() {
        auto udev_2 = flecs::udev::udev_t{};
        ASSERT_ANY_THROW(udev_2 = udev_1);
    }};

    t1.join();
}
