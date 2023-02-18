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

#pragma once

#include <cstdint>
#include <optional>
#include <string>

#include "util/udev/udev.h"

struct udev_hwdb;

namespace FLECS {
namespace udev {

class hwdb_t
{
public:
    hwdb_t();

    hwdb_t& operator=(hwdb_t other);

    hwdb_t(const hwdb_t& other);

    hwdb_t(hwdb_t&& other);

    ~hwdb_t();

    friend auto swap(hwdb_t& lhs, hwdb_t& rhs) //
        -> void;

    auto usb_vendor(std::uint16_t vid) //
        -> std::optional<std::string>;

    auto usb_device(std::uint16_t vid, std::uint16_t pid) //
        -> std::optional<std::string>;

private:
    udev_t _udev;
    udev_hwdb* _handle;
};

} // namespace udev
} // namespace FLECS
