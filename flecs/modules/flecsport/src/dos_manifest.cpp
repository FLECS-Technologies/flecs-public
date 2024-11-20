// Copyright 2021-2024 FLECS Technologies GmbH
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

#include "flecs/modules/flecsport/dos_manifest.h"

#include "flecs/util/json/json.h"

namespace flecs {

auto to_json(json_t& j, const dos_manifest_t& dos_manifest) //
    -> void
{
    j = json_t{
        {"_schemaVersion", dos_manifest.schema_version},
        {"time", dos_manifest.time},
        {"apps", dos_manifest.apps},
    };
}

auto from_json(const json_t& j, dos_manifest_t& dos_manifest) //
    -> void
{
    j.at("_schemaVersion").get_to(dos_manifest.schema_version);
    j.at("time").get_to(dos_manifest.time);
    j.at("apps").get_to(dos_manifest.apps);
}

auto to_json(json_t& j, const dos_app_t& dos_app) //
    -> void
{
    j = json_t({"name", dos_app.name});
    if (dos_app.version.has_value()) {
        j["version"] = dos_app.version.value();
    }
}

auto from_json(const json_t& j, dos_app_t& dos_app) //
    -> void
{
    j.at("name").get_to(dos_app.name);
    if (j.contains("version")) {
        std::string version;
        j.at("version").get_to(version);
        dos_app.version = std::move(version);
    }
}

} // namespace flecs