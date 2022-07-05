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

#include "util/usb/usb.h"

#include <libusb-1.0/libusb.h>

#include "util/sysfs/sysfs.h"
#include "util/udev/hwdb.h"

namespace FLECS {
namespace usb {

static auto __attribute__((constructor)) init() //
    -> void
{
    libusb_init(nullptr);
}

static auto __attribute__((destructor)) exit() //
    -> void
{
    libusb_exit(nullptr);
}

auto to_json(json_t& json, const device_t& device) //
    -> void
{
    json = json_t(
        {{"device", device.device},
         {"pid", device.pid},
         {"port", device.port},
         {"vendor", device.vendor},
         {"vid", device.vid}});
}

auto get_devices() //
    -> std::vector<device_t>
{
    constexpr auto NUM_USB_PORTS = 7;

    auto devices = std::vector<device_t>{};

    auto hwdb = udev::hwdb_t{};

    auto usb_devices = static_cast<libusb_device**>(nullptr);
    const auto device_count = libusb_get_device_list(nullptr, &usb_devices);

    devices.reserve(device_count);

    for (ssize_t i = 0; i < device_count; ++i)
    {
        auto desc = libusb_device_descriptor{};
        if (libusb_get_device_descriptor(usb_devices[i], &desc) != 0)
        {
            continue;
        }

        auto port = std::string{};

        std::uint8_t port_numbers[NUM_USB_PORTS] = {};
        const auto bus = libusb_get_bus_number(usb_devices[i]);
        const auto num_ports = libusb_get_port_numbers(usb_devices[i], port_numbers, NUM_USB_PORTS);

        if (num_ports == 0)
        {
            port = std::string{"usb"} + std::to_string(bus);
        }
        else
        {
            port = std::to_string(bus) + "-" + std::to_string(port_numbers[0]);
            for (auto i = 1; i < num_ports; ++i)
            {
                port += "." + std::to_string(port_numbers[i]);
            }
        }
        const auto vendor = hwdb.usb_vendor(desc.idVendor).has_value() ? hwdb.usb_vendor(desc.idVendor).value()
                            : sysfs::usb_vendor(port).has_value()      ? sysfs::usb_vendor(port).value()
                                                                       : std::string{};

        const auto device = hwdb.usb_device(desc.idVendor, desc.idProduct).has_value()
                                ? hwdb.usb_device(desc.idVendor, desc.idProduct).value()
                            : sysfs::usb_device(port).has_value() ? sysfs::usb_device(port).value()
                                                                  : std::string{};

        devices.emplace_back(device_t{
            .pid = desc.idProduct,
            .vid = desc.idVendor,
            .device = std::move(device),
            .port = std::move(port),
            .vendor = std::move(vendor)});
    }

    libusb_free_device_list(usb_devices, 1);

    return devices;
}

} // namespace usb
} // namespace FLECS
