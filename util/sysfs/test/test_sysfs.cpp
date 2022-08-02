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

#include <gtest/gtest.h>

#include <fstream>
#include <string>

#include "util/fs/fs.h"
#include "util/sysfs/sysfs.h"

constexpr auto port = "2-1";

#define USB_DEVICE "FLECS Test Device"
#define USB_VENDOR "FLECS Technologies GmbH"

TEST(sysfs, prepare)
{
    using std::operator""s;

    constexpr auto base_path = "flecs-sysfs/";
    const auto port_path = std::string{base_path}.append("2-1/");

    ASSERT_NO_THROW(FLECS::fs::remove_all(base_path));
    ASSERT_NO_THROW(FLECS::fs::create_directories(port_path));

    auto file_device = std::ofstream{port_path + "product"};
    file_device << USB_DEVICE;
    ASSERT_TRUE(file_device.good());

    auto file_vendor = std::ofstream{port_path + "manufacturer"};
    file_vendor << USB_VENDOR;
    ASSERT_TRUE(file_vendor.good());
}

TEST(sysfs, usb_device)
{
    const auto device = FLECS::sysfs::usb_device(port);

    ASSERT_TRUE(device.has_value());
    ASSERT_EQ(device.value(), USB_DEVICE);
}

TEST(sysfs, usb_vendor)
{
    const auto vendor = FLECS::sysfs::usb_vendor(port);

    ASSERT_TRUE(vendor.has_value());
    ASSERT_EQ(vendor.value(), USB_VENDOR);
}
