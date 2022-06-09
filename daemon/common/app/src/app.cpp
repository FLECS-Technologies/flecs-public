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

#include "app.h"

namespace FLECS {

app_t::app_t()
    : app_manifest_t{}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{}
    , _desired{}
{}

app_t::app_t(const std::filesystem::path& manifest_path, app_status_e status, app_status_e desired)
    : app_manifest_t{app_manifest_t::from_yaml_file(manifest_path)}
    , _license_key{}
    , _download_token{}
    , _installed_size{}
    , _status{status}
    , _desired{desired}
{
    if (!yaml_valid())
    {
        *this = app_t{};
    }
}

app_t::app_t(const std::string& manifest_string, app_status_e status, app_status_e desired)
    : app_manifest_t{app_manifest_t::from_yaml_string(manifest_string)}
    , _status{status}
    , _desired(desired)
{
    if (!yaml_valid())
    {
        *this = app_t{};
    }
}

void to_json(json_t& j, const app_t& app)
{
    const auto& parent = static_cast<const app_manifest_t&>(app);
    to_json(j, parent);
    j.push_back({"status", app_status_to_string(app._status)});
    j.push_back({"desired", app_status_to_string(app._desired)});
}

} // namespace FLECS
