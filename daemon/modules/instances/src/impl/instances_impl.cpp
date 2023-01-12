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

#include "impl/instances_impl.h"

#include "api/api.h"
#include "modules/factory/factory.h"
#include "modules/jobs/jobs.h"

namespace FLECS {
namespace impl {

module_instances_impl_t::module_instances_impl_t(module_instances_t* parent)
    : _parent{parent}
    , _jobs_api{}
{}

module_instances_impl_t::~module_instances_impl_t()
{}

auto module_instances_impl_t::do_init() //
    -> void
{
    _jobs_api = std::dynamic_pointer_cast<FLECS::module_jobs_t>(api::query_module("jobs"));
}

} // namespace impl
} // namespace FLECS
