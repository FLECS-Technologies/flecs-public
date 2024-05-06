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

#include "flecs/modules/deployments/deployments.h"

#include "flecs/modules/deployments/impl/deployments_impl.h"
#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {


deployments_t::deployments_t()
    : _impl{new impl::deployments_t{}}
{}

deployments_t::~deployments_t()
{}

auto deployments_t::do_init() //
    -> void
{}

auto deployments_t::do_deinit() //
    -> void
{}

} // namespace module
} // namespace flecs
