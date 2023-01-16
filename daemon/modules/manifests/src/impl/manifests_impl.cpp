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

#include "impl/manifests_impl.h"

namespace FLECS {
namespace impl {

module_manifests_t::module_manifests_t(FLECS::module_manifests_t* parent)
    : _parent{parent}
{}

module_manifests_t::~module_manifests_t()
{}

auto module_manifests_t::do_init() //
    -> void
{}

auto module_manifests_t::do_deinit() //
    -> void
{}

} // namespace impl
} // namespace FLECS
