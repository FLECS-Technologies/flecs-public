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

#include "flecs/modules/version/version.h"

#include <cstdio>

#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {

version_t::version_t()
{}

auto version_t::do_init() //
    -> void
{}

auto version_t::do_deinit() //
    -> void
{}

auto version_t::core_version() const //
    -> std::string
{
    return std::string{FLECS_VERSION} + "-" + FLECS_GIT_SHA;
}

auto version_t::api_version() const //
    -> std::string
{
    return FLECS_API_VERSION;
}

} // namespace module
} // namespace flecs
