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

#include "flecs/util/usb/usb.h"

#include <libusb-1.0/libusb.h>

#include <tuple>

#include "flecs/util/sysfs/sysfs.h"
#include "flecs/util/udev/hwdb.h"

namespace flecs {
namespace usb {

auto operator<=>(const device_t& lhs, const device_t& rhs) //
    -> std::strong_ordering
{
    return std::tie(lhs.vid, lhs.pid, lhs.port) <=> std::tie(rhs.vid, rhs.pid, rhs.port);
}

auto operator==(const device_t& lhs, const device_t& rhs) //
    -> bool
{
    return lhs <=> rhs == std::strong_ordering::equal;
}

auto operator!=(const device_t& lhs, const device_t& rhs) //
    -> bool
{
    return !(lhs == rhs);
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

auto from_json(const json_t& json, device_t& device) //
    -> void
{
    json.at("device").get_to(device.device);
    json.at("pid").get_to(device.pid);
    json.at("port").get_to(device.port);
    json.at("vendor").get_to(device.vendor);
    json.at("vid").get_to(device.vid);
}

auto get_devices() //
    -> std::set<device_t>
{
    constexpr auto NUM_USB_PORTS = 7;

    auto devices = std::set<device_t>{};

    auto hwdb = udev::hwdb_t{};

    libusb_context* context = nullptr;
    libusb_init(&context);

    auto usb_devices = static_cast<libusb_device**>(nullptr);
    const auto device_count = libusb_get_device_list(context, &usb_devices);

    for (ssize_t i = 0; i < device_count; ++i) {
        auto desc = libusb_device_descriptor{};
        if (libusb_get_device_descriptor(usb_devices[i], &desc) != 0) {
            continue;
        }

        auto port = std::string{};

        std::uint8_t port_numbers[NUM_USB_PORTS] = {};
        const auto bus = libusb_get_bus_number(usb_devices[i]);
        const auto num_ports = libusb_get_port_numbers(usb_devices[i], port_numbers, NUM_USB_PORTS);

        if (num_ports == 0) {
            port = std::string{"usb"} + std::to_string(bus);
        } else {
            port = std::to_string(bus) + "-" + std::to_string(port_numbers[0]);
            for (auto i = 1; i < num_ports; ++i) {
                port += "." + std::to_string(port_numbers[i]);
            }
        }
        auto vendor = hwdb.usb_vendor(desc.idVendor)
                          .value_or(sysfs::usb_vendor(port).value_or(
                              "Unknown vendor " + std::to_string(desc.idVendor)));

        auto device = hwdb.usb_device(desc.idVendor, desc.idProduct)
                          .value_or(sysfs::usb_device(port).value_or(
                              "Unknown device " + std::to_string(desc.idProduct)));
        devices.emplace(device_t{
            .vid = desc.idVendor,
            .pid = desc.idProduct,
            .port = std::move(port),
            .device = std::move(device),
            .vendor = std::move(vendor)});
    }

    libusb_free_device_list(usb_devices, 1);
    libusb_exit(context);
    context = nullptr;

    return devices;
}

} // namespace usb
} // namespace flecs
