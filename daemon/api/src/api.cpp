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

#include "api.h"

namespace FLECS {

flecs_api_t::flecs_api_t()
    : _app{}
    , _bp_v2{"v2"}
{
    _app.register_blueprint(_bp_v2);
}

flecs_api_t::~flecs_api_t()
{}

auto flecs_api_t::instance() noexcept //
    -> flecs_api_t&
{
    static auto instance = flecs_api_t{};
    return instance;
}

} // namespace FLECS
