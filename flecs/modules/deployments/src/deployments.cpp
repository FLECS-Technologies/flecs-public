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

auto deployments_t::do_load(const fs::path& base_path) //
    -> result_t
{
    return _impl->do_module_load(base_path);
}

auto deployments_t::do_init() //
    -> void
{}

auto deployments_t::do_deinit() //
    -> void
{}

auto deployments_t::query_deployment(std::string_view id) //
    -> std::shared_ptr<deployments::deployment_t>
{
    return _impl->do_query_deployment(std::move(id));
}

} // namespace module
} // namespace flecs
