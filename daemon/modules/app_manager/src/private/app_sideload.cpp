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

#include <cstdio>
#include <filesystem>

#include "app/app.h"
#include "private/app_manager_private.h"

namespace FLECS {
namespace Private {

http_status_e module_app_manager_private_t::do_sideload(const std::string& manifest_path, Json::Value& response)
{
    // Step 1: Parse transferred manifest
    auto app = app_t{manifest_path};
    if (!app.yaml_loaded())
    {
        response["additionalInfo"] = "Could not open manifest " + manifest_path;
        return http_status_e::InternalServerError;
    }

    // Step 2: Copy manifest to local storage
    const auto path = build_manifest_path(app.name(), app.version());

    std::error_code ec;
    std::filesystem::remove(path, ec);
    std::filesystem::copy(manifest_path, path, ec);
    if (ec)
    {
        response["additionalInfo"] = "Could not copy manifest to " + path;
        return http_status_e::InternalServerError;
    }

    // Step 3: Forward to manifest installation
    return do_install(manifest_path, response);
}

} // namespace Private
} // namespace FLECS