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

#include <string>
#include <vector>

#include "daemon/modules/apps/types/app_key.h"
#include "daemon/modules/instances/types/instance.h"
#include "util/json/json.h"
#include "util/sysinfo/sysinfo.h"

namespace flecs {

struct export_manifest_t
{
    explicit export_manifest_t(bool init = false);

    // generic info
    std::string time;
    // contents
    struct
    {
        std::vector<apps::key_t> apps;
        std::vector<instances::instance_t> instances;
    } contents;

    // device info
    struct
    {
        sysinfo_t sysinfo;
        std::string hostname;
    } device;

    // version
    struct
    {
        std::string core;
        std::string api;
    } version;
};

auto to_json(json_t& j, const export_manifest_t& export_manifest) //
    -> void;

auto from_json(const json_t& j, export_manifest_t& export_manifest) //
    -> void;

} // namespace flecs
