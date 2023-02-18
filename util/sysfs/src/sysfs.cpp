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

#include "util/sysfs/sysfs.h"

#include <fstream>

namespace FLECS {
namespace sysfs {

#ifndef FLECS_UNIT_TEST
constexpr auto base_path = "/sys/bus/usb/devices/";
#else
constexpr auto base_path = "flecs-sysfs/";
#endif

namespace {
auto read_file(std::string_view path) //
    -> std::optional<std::string>
{
    auto file = std::ifstream{path.data()};
    if (!file.good()) {
        return {};
    }

    auto line = std::string{};
    std::getline(file, line);
    return line;
}
} // namespace

auto usb_vendor(std::string_view port) //
    -> std::optional<std::string>
{
    const auto path = std::string{base_path}.append(port).append("/manufacturer");
    return read_file(path);
}

auto usb_device(std::string_view port) //
    -> std::optional<std::string>
{
    const auto path = std::string{base_path}.append(port).append("/product");
    return read_file(path);
}

auto usb_busnum(std::string_view port) //
    -> std::optional<std::uint16_t>
{
    const auto path = std::string{base_path}.append(port).append("/busnum");
    const auto busnum = read_file(path);
    if (busnum.has_value()) {
        return std::stoi(busnum.value());
    }
    return {};
}

auto usb_devnum(std::string_view port) //
    -> std::optional<std::uint16_t>
{
    const auto path = std::string{base_path}.append(port).append("/devnum");
    const auto devnum = read_file(path);
    if (devnum.has_value()) {
        return std::stoi(devnum.value());
    }
    return {};
}

} // namespace sysfs
} // namespace FLECS
