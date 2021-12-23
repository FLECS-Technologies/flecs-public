// Copyright 2021 FLECS Technologies GmbH
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

#ifndef FLECS_service_service_table_h
#define FLECS_service_service_table_h

#include <memory>

#include "service_app_manager.h"
#include "service_factory.h"
#include "service_help.h"
#include "service_rpc.h"
#include "util/container/map_constexpr.h"

namespace FLECS {

using make_service_t = std::shared_ptr<service_t> (*)();
using make_service_table_t = FLECS::map_c<const char*, make_service_t, 3, string_comparator>;

constexpr FLECS::make_service_table_t make_service_table = {{
    std::make_pair<const char*, make_service_t>("app-manager", &FLECS::make_service<FLECS::service_app_manager_t>),
    std::make_pair<const char*, make_service_t>("help", &FLECS::make_service<FLECS::service_help>),
    std::make_pair<const char*, make_service_t>("rpc", &FLECS::make_service<FLECS::service_rpc>),
}};

} // namespace FLECS

#endif // FLECS_service_service_table_h
