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

#ifndef D79653EB_6825_4DCA_A8AD_CFC77A04CACF
#define D79653EB_6825_4DCA_A8AD_CFC77A04CACF

#include <cstdint>
#include <optional>
#include <string>

namespace FLECS {
namespace sysfs {

auto usb_vendor(std::string_view port) //
    -> std::optional<std::string>;

auto usb_device(std::string_view port) //
    -> std::optional<std::string>;

auto usb_busnum(std::string_view port) //
    -> std::optional<std::uint16_t>;

auto usb_devnum(std::string_view port) //
    -> std::optional<std::uint16_t>;

} // namespace sysfs
} // namespace FLECS

#endif /* D79653EB_6825_4DCA_A8AD_CFC77A04CACF */
