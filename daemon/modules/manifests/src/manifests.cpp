// Copyright 2021-2022 FLECS Technologies GmbH
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

#include "manifests.h"

#include "factory/factory.h"
#include "impl/manifests_impl.h"

namespace FLECS {

namespace {
register_module_t<module_manifests_t> _reg("manifests");
}

module_manifests_t::module_manifests_t()
    : _impl{new impl::module_manifests_t{this}}
{}

module_manifests_t::~module_manifests_t()
{}

auto module_manifests_t::do_init() //
    -> void
{
    return _impl->do_init();
}

auto module_manifests_t::do_deinit() //
    -> void
{
    return _impl->do_deinit();
}

} // namespace FLECS
