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

#include <fstream>
#include <string>

#include "util/fs/fs.h"
#include "util/sysfs/sysfs.h"

constexpr auto port = "2-1";
constexpr auto port_invalid = "2-3";

#define USB_DEVICE "FLECS Test Device"
#define USB_VENDOR "FLECS Technologies GmbH"
#define USB_BUSNUM 3
#define USB_DEVNUM 19

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

    auto file_busnum = std::ofstream{port_path + "busnum"};
    file_busnum << USB_BUSNUM;
    ASSERT_TRUE(file_busnum.good());

    auto file_devnum = std::ofstream{port_path + "devnum"};
    file_devnum << USB_DEVNUM;
    ASSERT_TRUE(file_devnum.good());
}

TEST(sysfs, usb_device)
{
    const auto device = FLECS::sysfs::usb_device(port);

    ASSERT_TRUE(device.has_value());
    ASSERT_EQ(device.value(), USB_DEVICE);

    const auto device_invalid = FLECS::sysfs::usb_device(port_invalid);
    ASSERT_FALSE(device_invalid.has_value());
}

TEST(sysfs, usb_vendor)
{
    const auto vendor = FLECS::sysfs::usb_vendor(port);

    ASSERT_TRUE(vendor.has_value());
    ASSERT_EQ(vendor.value(), USB_VENDOR);

    const auto vendor_invalid = FLECS::sysfs::usb_vendor(port_invalid);
    ASSERT_FALSE(vendor_invalid.has_value());
}

TEST(sysfs, usb_busnum)
{
    const auto busnum = FLECS::sysfs::usb_busnum(port);

    ASSERT_TRUE(busnum.has_value());
    ASSERT_EQ(busnum.value(), USB_BUSNUM);

    const auto busnum_invalid = FLECS::sysfs::usb_busnum(port_invalid);
    ASSERT_FALSE(busnum_invalid.has_value());
}

TEST(sysfs, usb_devnum)
{
    const auto devnum = FLECS::sysfs::usb_devnum(port);

    ASSERT_TRUE(devnum.has_value());
    ASSERT_EQ(devnum.value(), USB_DEVNUM);

    const auto devnum_invalid = FLECS::sysfs::usb_devnum(port_invalid);
    ASSERT_FALSE(devnum_invalid.has_value());
}
