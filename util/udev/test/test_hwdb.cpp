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

#include "util/udev/hwdb.h"

// vid of Linux Foundation
#define VID_LINUX 0x1d6b
// invalid vid
#define VID_INVALID 0xffff

// pid of USB 2.0 root hub
#define PID_ROOT_HUB_2 0002
// invalid pid
#define PID_INVALID 0xffff

TEST(hwdb, init)
{
    auto hwdb_1 = FLECS::udev::hwdb_t{};
    auto hwdb_2 = FLECS::udev::hwdb_t{};
    auto hwdb_3 = FLECS::udev::hwdb_t{};

    ASSERT_NO_THROW((hwdb_2 = hwdb_1));
    ASSERT_NO_THROW((hwdb_2 = FLECS::udev::hwdb_t{hwdb_1}));
    ASSERT_NO_THROW((hwdb_2 = FLECS::udev::hwdb_t{std::move(hwdb_1)}));
    ASSERT_NO_THROW((hwdb_3 = std::move(hwdb_2)));
}

TEST(hwdb, vendor)
{
    auto hwdb = FLECS::udev::hwdb_t{};

    const auto vid_1 = VID_LINUX;
    const auto vid_2 = VID_INVALID;

    const auto vendor_1 = hwdb.usb_vendor(vid_1);
    ASSERT_TRUE(vendor_1.has_value());
    ASSERT_EQ(vendor_1.value(), "Linux Foundation");

    const auto vendor_2 = hwdb.usb_vendor(vid_2);
    ASSERT_FALSE(vendor_2.has_value());
}

TEST(hwdb, model)
{
    auto hwdb = FLECS::udev::hwdb_t{};

    const auto vid = VID_LINUX;
    const auto pid_1 = PID_ROOT_HUB_2;
    const auto pid_2 = PID_INVALID;

    const auto device_1 = hwdb.usb_device(vid, pid_1);
    ASSERT_TRUE(device_1.has_value());
    ASSERT_EQ(device_1.value(), "2.0 root hub");

    const auto device_2 = hwdb.usb_device(vid, pid_2);
    ASSERT_FALSE(device_2.has_value());
}
