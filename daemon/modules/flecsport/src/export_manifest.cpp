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

#include "export_manifest.h"

#include <unistd.h>

#include "modules/factory/factory.h"
#include "modules/version/version.h"

namespace FLECS {

export_manifest_t::export_manifest_t(bool init)
    : time{}
    , contents{}
    , device{}
    , version{}
{
    if (init) {
        char hostname[HOST_NAME_MAX + 1] = {};
        gethostname(hostname, HOST_NAME_MAX);
        device.hostname = hostname;

        version.core = std::dynamic_pointer_cast<module_version_t>(api::query_module("version"))
                           ->core_version();
        version.api = std::dynamic_pointer_cast<module_version_t>(api::query_module("version"))
                          ->api_version();
    }
}

auto to_json(json_t& j, const export_manifest_t& export_manifest) //
    -> void
{
    j["_schemaVersion"] = "2.0.0";
    j["time"] = export_manifest.time;
    j["contents"]["apps"] = export_manifest.contents.apps;
    j["contents"]["instances"] = export_manifest.contents.instances;
    j["device"]["sysinfo"] = export_manifest.device.sysinfo;
    j["device"]["hostname"] = export_manifest.device.hostname;
    j["version"]["core"] = export_manifest.version.core;
    j["version"]["api"] = export_manifest.version.api;
}

auto from_json(const json_t& j, export_manifest_t& export_manifest) //
    -> void
{
    try {
        j.at("time").get_to(export_manifest.time);
        j.at("contents").at("apps").get_to(export_manifest.contents.apps);
        j.at("contents").at("instances").get_to(export_manifest.contents.instances);
        j.at("device").at("sysinfo").get_to(export_manifest.device.sysinfo);
        j.at("device").at("hostname").get_to(export_manifest.device.hostname);
        j.at("version").at("core").get_to(export_manifest.version.core);
        j.at("version").at("api").get_to(export_manifest.version.api);
    } catch (...) {
        export_manifest = export_manifest_t{};
    }
}

} // namespace FLECS
