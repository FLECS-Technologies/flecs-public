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

#ifndef FLECS_service_app_status_h
#define FLECS_service_app_status_h

#include <string>

#include "util/container/map_constexpr.h"

namespace FLECS {

enum app_status_e : char
{
    NOT_INSTALLED = 'n',
    MANIFEST_DOWNLOADED = 'm',
    IMAGE_DOWNLOADED = 'd',
    INSTALLED = 'i',
    REMOVED = 'r',
    PURGED = 'p',
};

inline std::string to_string(app_status_e val)
{
    auto res = std::string{};
    return res.append(1, val);
}

using app_status_to_string_t = map_c<app_status_e, const char*, 6>;
constexpr app_status_to_string_t app_status_to_string_table = {{
    std::make_pair(app_status_e::NOT_INSTALLED, "not installed"),
    std::make_pair(app_status_e::MANIFEST_DOWNLOADED, "manifest downloaded"),
    std::make_pair(app_status_e::IMAGE_DOWNLOADED, "image downloaded"),
    std::make_pair(app_status_e::INSTALLED, "installed"),
    std::make_pair(app_status_e::REMOVED, "removed"),
    std::make_pair(app_status_e::PURGED, "purged"),
}};

constexpr const char* app_status_to_string(app_status_e status)
{
    if (app_status_to_string_table.find(status) != app_status_to_string_table.end())
    {
        return app_status_to_string_table.at(status).second;
    }

    return "unknown";
}

} // namespace FLECS

#endif // FLECS_service_app_status_h
