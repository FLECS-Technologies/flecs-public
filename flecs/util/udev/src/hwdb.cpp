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

#include "flecs/util/udev/hwdb.h"

#include <libudev.h>

#include <cstring>

namespace flecs {
namespace udev {

hwdb_t::hwdb_t()
    : _udev{}
    , _handle{udev_hwdb_new(*_udev)}
{}

hwdb_t::hwdb_t(const hwdb_t& other)
    : _udev{other._udev}
    , _handle{udev_hwdb_ref(other._handle)}
{}

hwdb_t::hwdb_t(hwdb_t&& other)
    : _udev{}
    , _handle{}
{
    swap(*this, other);
}

hwdb_t& hwdb_t::operator=(hwdb_t other)
{
    swap(*this, other);
    return *this;
}

hwdb_t::~hwdb_t()
{
    if (_handle) {
        udev_hwdb_unref(_handle);
    }
}

auto swap(hwdb_t& lhs, hwdb_t& rhs) //
    -> void
{
    using std::swap;
    swap(lhs._udev, rhs._udev);
    swap(lhs._handle, rhs._handle);
}

auto hwdb_t::usb_vendor(std::uint16_t vid) //
    -> std::optional<std::string>
{
    char modalias[11] = "usb:v0000*";

    std::snprintf(modalias, sizeof(modalias), "usb:v%04X*", vid);

    auto udev_entry = static_cast<udev_list_entry*>(nullptr);
    udev_list_entry_foreach(udev_entry, udev_hwdb_get_properties_list_entry(_handle, modalias, 0))
    {
        auto name = udev_list_entry_get_name(udev_entry);
        if (std::strcmp(name, "ID_VENDOR_FROM_DATABASE") == 0) {
            return udev_list_entry_get_value(udev_entry);
        }
    }

    return {};
}

auto hwdb_t::usb_device(std::uint16_t vid, std::uint16_t pid) //
    -> std::optional<std::string>
{
    char modalias[16] = "usb:v0000p0000*";

    std::snprintf(modalias, sizeof(modalias), "usb:v%04Xp%04X", vid, pid);

    auto udev_entry = static_cast<udev_list_entry*>(nullptr);

    udev_list_entry_foreach(udev_entry, udev_hwdb_get_properties_list_entry(_handle, modalias, 0))
    {
        auto model = udev_list_entry_get_name(udev_entry);
        if (std::strcmp(model, "ID_MODEL_FROM_DATABASE") == 0) {
            return udev_list_entry_get_value(udev_entry);
        }
    }

    return {};
}

} // namespace udev
} // namespace flecs
