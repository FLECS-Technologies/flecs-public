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
#include <string_view>

namespace flecs {
namespace instances {

enum class status_e {
    NotCreated,
    Requested,
    ResourcesReady,
    Created,
    Stopped,
    Running,
    Orphaned,
    Unknown,
};

auto to_string_view(const status_e& instance_status) //
    -> std::string_view;

auto to_string(const status_e& instance_status) //
    -> std::string;

auto status_from_string(std::string_view str) //
    -> status_e;

} // namespace instances
} // namespace flecs
