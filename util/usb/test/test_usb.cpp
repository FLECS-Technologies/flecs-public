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

#include "util/usb/usb.h"

#define USB_PID 0x1234
#define USB_PID_2 0x1235
#define USB_VID 0xabcd
#define USB_VID_2 0xabce
#define USB_DEVICE "FLECS Test Device"
#define USB_PORT "2.1-1"
#define USB_PORT_2 "2.1-2"
#define USB_VENDOR "FLECS Technologies GmbH"

TEST(usb, compare)
{
    const auto usb_device_1 = FLECS::usb::device_t{
        .pid = USB_PID,
        .vid = USB_VID,
        .device = USB_DEVICE,
        .port = USB_PORT,
        .vendor = USB_VENDOR,
    };
    // increment vid
    const auto usb_device_2 = FLECS::usb::device_t{
        .pid = USB_PID,
        .vid = USB_VID_2,
        .device = USB_DEVICE,
        .port = USB_PORT,
        .vendor = USB_VENDOR,
    };
    // increment pid
    const auto usb_device_3 = FLECS::usb::device_t{
        .pid = USB_PID_2,
        .vid = USB_VID,
        .device = USB_DEVICE,
        .port = USB_PORT,
        .vendor = USB_VENDOR,
    };
    // increment port
    const auto usb_device_4 = FLECS::usb::device_t{
        .pid = USB_PID,
        .vid = USB_VID,
        .device = USB_DEVICE,
        .port = USB_PORT_2,
        .vendor = USB_VENDOR,
    };

    ASSERT_TRUE(usb_device_1 == usb_device_1);
    ASSERT_FALSE(usb_device_1 != usb_device_1);
    ASSERT_FALSE(usb_device_1 < usb_device_1);
    ASSERT_TRUE(usb_device_1 <= usb_device_1);
    ASSERT_FALSE(usb_device_1 > usb_device_1);
    ASSERT_TRUE(usb_device_1 >= usb_device_1);

    ASSERT_FALSE(usb_device_1 == usb_device_2);
    ASSERT_TRUE(usb_device_1 != usb_device_2);
    ASSERT_TRUE(usb_device_1 < usb_device_2);
    ASSERT_TRUE(usb_device_1 <= usb_device_2);
    ASSERT_FALSE(usb_device_1 > usb_device_2);
    ASSERT_FALSE(usb_device_1 >= usb_device_2);

    ASSERT_FALSE(usb_device_1 == usb_device_3);
    ASSERT_TRUE(usb_device_1 != usb_device_3);
    ASSERT_TRUE(usb_device_1 < usb_device_3);
    ASSERT_TRUE(usb_device_1 <= usb_device_3);
    ASSERT_FALSE(usb_device_1 > usb_device_3);
    ASSERT_FALSE(usb_device_1 >= usb_device_3);

    ASSERT_FALSE(usb_device_1 == usb_device_4);
    ASSERT_TRUE(usb_device_1 != usb_device_4);
    ASSERT_TRUE(usb_device_1 < usb_device_4);
    ASSERT_TRUE(usb_device_1 <= usb_device_4);
    ASSERT_FALSE(usb_device_1 > usb_device_4);
    ASSERT_FALSE(usb_device_1 >= usb_device_4);

    ASSERT_FALSE(usb_device_3 == usb_device_2);
    ASSERT_TRUE(usb_device_3 != usb_device_2);
    ASSERT_TRUE(usb_device_3 < usb_device_2);
    ASSERT_TRUE(usb_device_3 <= usb_device_2);
    ASSERT_FALSE(usb_device_3 > usb_device_2);
    ASSERT_FALSE(usb_device_3 >= usb_device_2);

    ASSERT_FALSE(usb_device_4 == usb_device_3);
    ASSERT_TRUE(usb_device_4 != usb_device_3);
    ASSERT_TRUE(usb_device_4 < usb_device_3);
    ASSERT_TRUE(usb_device_4 <= usb_device_3);
    ASSERT_FALSE(usb_device_4 > usb_device_3);
    ASSERT_FALSE(usb_device_4 >= usb_device_3);
}

TEST(usb, to_json)
{
    const auto usb_device = FLECS::usb::device_t{
        .pid = USB_PID,
        .vid = USB_VID,
        .device = USB_DEVICE,
        .port = USB_PORT,
        .vendor = USB_VENDOR};

    auto json = FLECS::json_t{};
    to_json(json, usb_device);

    ASSERT_TRUE(FLECS::is_valid_json(json));
    ASSERT_EQ(json["pid"], USB_PID);
    ASSERT_EQ(json["vid"], USB_VID);
    ASSERT_EQ(json["device"], USB_DEVICE);
    ASSERT_EQ(json["port"], USB_PORT);
    ASSERT_EQ(json["vendor"], USB_VENDOR);
}

TEST(usb, from_json)
{
    const auto json_string =
        R"({"pid":4660,"vid":43981,"device":"FLECS Test Device","port":"2.1-1","vendor":"FLECS Technologies GmbH"})";
    const auto json = FLECS::parse_json(json_string);

    auto usb_device = FLECS::usb::device_t{};
    from_json(json, usb_device);

    ASSERT_TRUE(FLECS::is_valid_json(json));
    ASSERT_EQ(usb_device.pid, USB_PID);
    ASSERT_EQ(usb_device.vid, USB_VID);
    ASSERT_EQ(usb_device.device, USB_DEVICE);
    ASSERT_EQ(usb_device.port, USB_PORT);
    ASSERT_EQ(usb_device.vendor, USB_VENDOR);
}
