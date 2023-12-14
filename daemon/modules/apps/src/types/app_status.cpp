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

#include "daemon/modules/apps/types/app_status.h"

#include <algorithm>
#include <array>
#include <tuple>

namespace flecs {
namespace apps {

static constexpr auto strings = std::array<std::tuple<status_e, std::string_view>, 8>{{
    {status_e::NotInstalled, "not installed"},
    {status_e::ManifestDownloaded, "manifest downloaded"},
    {status_e::TokenAcquired, "token acquired"},
    {status_e::ImageDownloaded, "image downloaded"},
    {status_e::Installed, "installed"},
    {status_e::Removed, "removed"},
    {status_e::Purged, "purged"},
    {status_e::Orphaned, "orphaned"},
}};

auto to_string_view(status_e app_status) //
    -> std::string_view
{
    const auto it = std::find_if(
        strings.cbegin(),
        strings.cend(),
        [&app_status](decltype(strings)::const_reference elem) { return std::get<0>(elem) == app_status; });

    return it == strings.cend() ? "unknown" : std::get<1>(*it);
}

auto to_string(status_e app_status) //
    -> std::string
{
    return std::string{to_string_view(app_status)};
}

auto status_from_string(std::string_view str) //
    -> status_e
{
    const auto it =
        std::find_if(strings.cbegin(), strings.cend(), [&str](decltype(strings)::const_reference elem) {
            return std::get<1>(elem) == str;
        });

    return it == strings.cend() ? status_e::Unknown : std::get<0>(*it);
}

} // namespace apps
} // namespace flecs
