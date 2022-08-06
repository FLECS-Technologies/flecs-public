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

#include <thread>

#include "util/usb/usb.h"

#define USB_ADDR 0x0001
#define USB_PID 0x1234
#define USB_VID 0xabcd
#define USB_DEVICE "FLECS Test Device"
#define USB_PORT "2.1-1"
#define USB_VENDOR "FLECS Technologies GmbH"

TEST(usb, to_json)
{
    const auto usb_device = FLECS::usb::device_t{
        .addr = USB_ADDR,
        .pid = USB_PID,
        .vid = USB_VID,
        .device = USB_DEVICE,
        .port = USB_PORT,
        .vendor = USB_VENDOR};

    auto json = FLECS::json_t{};
    to_json(json, usb_device);

    ASSERT_TRUE(FLECS::is_valid_json(json));
    ASSERT_EQ(json["addr"], USB_ADDR);
    ASSERT_EQ(json["pid"], USB_PID);
    ASSERT_EQ(json["vid"], USB_VID);
    ASSERT_EQ(json["device"], USB_DEVICE);
    ASSERT_EQ(json["port"], USB_PORT);
    ASSERT_EQ(json["vendor"], USB_VENDOR);
}
