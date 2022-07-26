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

#include "app_status.h"

#include <map>

namespace FLECS {

auto to_string(app_status_e app_status) //
    -> std::string
{
    const auto strings = std::map<app_status_e, std::string>{
        {app_status_e::NOT_INSTALLED, "not installed"},
        {app_status_e::MANIFEST_DOWNLOADED, "manifest downloaded"},
        {app_status_e::TOKEN_ACQUIRED, "token acquired"},
        {app_status_e::IMAGE_DOWNLOADED, "image downloaded"},
        {app_status_e::INSTALLED, "installed"},
        {app_status_e::REMOVED, "removed"},
        {app_status_e::PURGED, "purged"},
    };

    return strings.count(app_status) ? strings.at(app_status) : "unknown";
}

auto app_status_from_string(std::string_view str) //
    -> app_status_e
{
    const auto status = std::map<std::string_view, app_status_e>{
        {"not installed", app_status_e::NOT_INSTALLED},
        {"manifest downloaded", app_status_e::MANIFEST_DOWNLOADED},
        {"token acquired", app_status_e::TOKEN_ACQUIRED},
        {"image downloaded", app_status_e::IMAGE_DOWNLOADED},
        {"installed", app_status_e::INSTALLED},
        {"removed", app_status_e::REMOVED},
        {"purged", app_status_e::PURGED},
    };

    return status.count(str) ? status.at(str) : app_status_e::UNKNOWN;
}

} // namespace FLECS
