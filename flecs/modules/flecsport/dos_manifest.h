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

#pragma once

#include <string>
#include <vector>

#include "flecs/modules/apps/types/app_key.h"
#include "flecs/modules/instances/types/instance.h"
#include "flecs/util/json/json.h"
#include "flecs/util/sysinfo/sysinfo.h"

namespace flecs {

struct dos_app_t
{
    std::string name;
    std::optional<std::string> version;
};

struct dos_manifest_t
{
    std::string time;
    std::string schema_version;
    std::vector<dos_app_t> apps;
};

auto to_json(json_t& j, const dos_manifest_t& dos_manifest) //
    -> void;

auto from_json(const json_t& j, dos_manifest_t& dos_manifest) //
    -> void;

auto to_json(json_t& j, const dos_app_t& dos_app) //
    -> void;

auto from_json(const json_t& j, dos_app_t& dos_app) //
    -> void;

} // namespace flecs
